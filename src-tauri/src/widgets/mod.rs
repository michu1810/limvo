//! The Limvo card on the Windows 11 Widgets board.
//!
//! The board shows widgets of packaged apps only, so this runs in the
//! Microsoft Store build alone. `limvo.exe` is its own widget provider: at
//! start-up it registers a COM class object for [`CLSID`], which the package
//! manifest names as the provider. When the board needs the provider while
//! Limvo is not running, Windows starts Limvo, which then registers itself as
//! usual. The interface owns the translations and the pins, so the flyout
//! window sends the card's content ([`publish`]); the provider only hands it
//! to the board with a fixed template.

#[cfg(windows)]
#[allow(
    clippy::pedantic,
    missing_debug_implementations,
    unsafe_op_in_unsafe_fn,
    unused_qualifications
)]
mod bindings;

use serde::{Deserialize, Serialize};

/// The provider's COM class, named in `msix/AppxManifest.xml`. Never change it:
/// pinned widgets find their provider by it.
#[cfg(windows)]
const CLSID: windows_core::GUID =
    windows_core::GUID::from_u128(0x3754_9728_fbb6_4975_ae5e_047f_0ba9_d0a2);

/// The card's template, bound to [`CardData`] by the board.
const TEMPLATE: &str = include_str!("template.json");

/// One limit on the card, already worded in the user's language.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CardItem {
    /// The service, and the account for an extra one: "Claude · Work".
    pub name: String,
    /// "16%".
    pub value: String,
    /// "until Tue · Running hot".
    pub detail: String,
    /// Running hot or spent, for the warning colour.
    pub hot: bool,
}

/// What the card shows: the main limit for the small size, more for the
/// larger ones. The flyout chooses and words them; the provider passes them on.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CardData {
    /// Up to six limits, the pinned ones in their order.
    pub items: Vec<CardItem>,
    /// Shown instead of the limits when there are none: "Sign in to a service".
    pub empty: String,
    /// The refresh button's label.
    pub refresh: String,
}

impl CardData {
    /// Keeps the content within what the board can show, whatever the window sent.
    fn bounded(mut self) -> Self {
        fn cut(text: &mut String, limit: usize) {
            if let Some((index, _)) = text.char_indices().nth(limit) {
                text.truncate(index);
            }
        }
        self.items.truncate(6);
        for item in &mut self.items {
            cut(&mut item.name, 40);
            cut(&mut item.value, 12);
            cut(&mut item.detail, 60);
        }
        cut(&mut self.empty, 120);
        cut(&mut self.refresh, 30);
        self
    }

    /// The data the template binds to: the first limit for the small size,
    /// three for the medium one and all of them for the large one.
    fn to_json(&self) -> String {
        let take = |count: usize| &self.items[..self.items.len().min(count)];
        serde_json::json!({
            "small": self.items.first(),
            "medium": take(3),
            "large": take(6),
            "hasItems": !self.items.is_empty(),
            "empty": self.empty,
            "refresh": self.refresh,
        })
        .to_string()
    }
}

#[cfg(windows)]
pub use platform::{publish, start};

#[cfg(not(windows))]
pub fn start(_app: &tauri::AppHandle) {}

#[cfg(not(windows))]
pub fn publish(_data: CardData) {}

#[cfg(windows)]
// `#[implement]` expands to code these two pedantic lints flag.
#[allow(clippy::inline_always, clippy::ref_as_ptr)]
mod platform {
    use std::sync::{Mutex, OnceLock, PoisonError};

    use tauri::{AppHandle, Manager};
    use windows::Win32::Foundation::{CLASS_E_NOAGGREGATION, E_POINTER};
    use windows::Win32::System::Com::{
        CLSCTX_LOCAL_SERVER, COINIT_MULTITHREADED, CoInitializeEx, CoRegisterClassObject,
        IClassFactory, IClassFactory_Impl, REGCLS_MULTIPLEUSE,
    };
    use windows::core::BOOL;
    use windows_core::{GUID, HSTRING, IUnknown, Interface, Ref, implement};

    use super::bindings::{
        IWidgetProvider, IWidgetProvider_Impl, WidgetActionInvokedArgs, WidgetContext,
        WidgetContextChangedArgs, WidgetManager, WidgetUpdateRequestOptions,
    };
    use super::{CLSID, CardData, TEMPLATE};
    use crate::{AppState, flyout, package};

    static APP: OnceLock<AppHandle> = OnceLock::new();
    /// The latest content, for widgets the board asks about before the next reading.
    static DATA: Mutex<Option<CardData>> = Mutex::new(None);

    /// Registers the provider with COM, in the Store build only. COM keeps the
    /// class object on a thread of its multithreaded apartment for the whole
    /// session; calls from the board arrive on COM's own threads.
    pub fn start(app: &AppHandle) {
        if !package::is_packaged() || APP.set(app.clone()).is_err() {
            return;
        }
        let spawned = std::thread::Builder::new()
            .name("widgets".into())
            .spawn(|| {
                // SAFETY: plain COM initialisation of this thread, kept for its lifetime.
                if let Err(error) = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.ok() {
                    log::warn!("widgets: COM did not start: {error}");
                    return;
                }
                let factory: IClassFactory = Factory.into();
                // SAFETY: the factory is a valid class object; the cookie is never
                // revoked, since the registration lasts as long as the process.
                match unsafe {
                    CoRegisterClassObject(&CLSID, &factory, CLSCTX_LOCAL_SERVER, REGCLS_MULTIPLEUSE)
                } {
                    Ok(_) => log::info!("widgets: provider registered"),
                    Err(error) => {
                        log::warn!("widgets: provider not registered: {error}");
                        return;
                    }
                }
                // The apartment, and with it the registration, lives while this thread does.
                loop {
                    std::thread::park();
                }
            });
        if let Err(error) = spawned {
            log::warn!("widgets: no thread for the provider: {error}");
        }
    }

    /// New content from the flyout: kept for widgets pinned later, and sent to
    /// every widget on the board now.
    pub fn publish(data: CardData) {
        if APP.get().is_none() {
            return;
        }
        let data = data.bounded();
        *DATA.lock().unwrap_or_else(PoisonError::into_inner) = Some(data);
        if let Err(error) = update_all() {
            log::warn!("widgets: update failed: {error}");
        }
    }

    fn update_all() -> windows_core::Result<()> {
        let manager = WidgetManager::GetDefault()?;
        for id in manager.GetWidgetIds()?.iter() {
            update(&manager, id)?;
        }
        Ok(())
    }

    fn update(manager: &WidgetManager, id: &HSTRING) -> windows_core::Result<()> {
        let data = DATA
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
            .unwrap_or_default();
        let options = WidgetUpdateRequestOptions::CreateInstance(id)?;
        options.SetTemplate(&HSTRING::from(TEMPLATE))?;
        options.SetData(&HSTRING::from(data.to_json()))?;
        manager.UpdateWidget(&options)
    }

    fn update_one(context: &WidgetContext) -> windows_core::Result<()> {
        update(&WidgetManager::GetDefault()?, &context.Id()?)
    }

    #[implement(IClassFactory)]
    struct Factory;

    impl IClassFactory_Impl for Factory_Impl {
        fn CreateInstance(
            &self,
            outer: Ref<'_, IUnknown>,
            iid: *const GUID,
            object: *mut *mut core::ffi::c_void,
        ) -> windows_core::Result<()> {
            if object.is_null() || iid.is_null() {
                return Err(E_POINTER.into());
            }
            // SAFETY: COM passes a valid out pointer, checked for null above.
            unsafe { *object = std::ptr::null_mut() };
            if outer.is_some() {
                return Err(CLASS_E_NOAGGREGATION.into());
            }
            let provider: IWidgetProvider = Provider.into();
            // SAFETY: `iid` and `object` come from COM and were checked for null.
            unsafe { provider.query(iid, object) }.ok()
        }

        fn LockServer(&self, _lock: BOOL) -> windows_core::Result<()> {
            Ok(())
        }
    }

    /// Answers the board. Every widget shows the same content, so the
    /// provider keeps no state of its own: it sends the latest content
    /// whenever the board shows interest.
    #[implement(IWidgetProvider)]
    struct Provider;

    impl IWidgetProvider_Impl for Provider_Impl {
        fn CreateWidget(&self, context: Ref<'_, WidgetContext>) -> windows_core::Result<()> {
            context.ok().and_then(update_one)
        }

        fn DeleteWidget(&self, _id: &HSTRING, _state: &HSTRING) -> windows_core::Result<()> {
            Ok(())
        }

        fn OnActionInvoked(
            &self,
            args: Ref<'_, WidgetActionInvokedArgs>,
        ) -> windows_core::Result<()> {
            let verb = args.ok()?.Verb()?;
            let Some(app) = APP.get() else {
                return Ok(());
            };
            if verb == "refresh" {
                if let Some(state) = app.try_state::<AppState>() {
                    state.poller.refresh_now();
                }
            } else if verb == "open" {
                let app = app.clone();
                // Windows move on the main thread.
                let _ = app
                    .clone()
                    .run_on_main_thread(move || flyout::show(&app, None));
            }
            Ok(())
        }

        fn OnWidgetContextChanged(
            &self,
            args: Ref<'_, WidgetContextChangedArgs>,
        ) -> windows_core::Result<()> {
            update_one(&args.ok()?.WidgetContext()?)
        }

        fn Activate(&self, context: Ref<'_, WidgetContext>) -> windows_core::Result<()> {
            context.ok().and_then(update_one)
        }

        fn Deactivate(&self, _id: &HSTRING) -> windows_core::Result<()> {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(name: &str) -> CardItem {
        CardItem {
            name: name.to_owned(),
            value: "16%".to_owned(),
            detail: "until Tue".to_owned(),
            hot: false,
        }
    }

    #[test]
    fn each_size_gets_its_share_of_the_limits() {
        let data = CardData {
            items: (0..5).map(|n| item(&n.to_string())).collect(),
            ..CardData::default()
        };
        let json: serde_json::Value = serde_json::from_str(&data.to_json()).unwrap();
        assert_eq!(json["small"]["name"], "0");
        assert_eq!(json["medium"].as_array().unwrap().len(), 3);
        assert_eq!(json["large"].as_array().unwrap().len(), 5);
        assert_eq!(json["hasItems"], true);
    }

    #[test]
    fn keeps_whatever_the_window_sends_within_bounds() {
        let data = CardData {
            items: (0..9).map(|_| item(&"x".repeat(100))).collect(),
            empty: "é".repeat(500),
            ..CardData::default()
        }
        .bounded();
        assert_eq!(data.items.len(), 6);
        assert_eq!(data.items[0].name.chars().count(), 40);
        assert_eq!(data.empty.chars().count(), 120);
    }

    #[test]
    fn the_template_is_an_adaptive_card() {
        let template: serde_json::Value = serde_json::from_str(TEMPLATE).unwrap();
        assert_eq!(template["type"], "AdaptiveCard");
    }

    #[test]
    fn an_empty_card_says_so() {
        let json: serde_json::Value = serde_json::from_str(&CardData::default().to_json()).unwrap();
        assert_eq!(json["hasItems"], false);
        assert!(json["small"].is_null());
    }
}
