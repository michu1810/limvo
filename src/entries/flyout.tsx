import "../styles/flyout.css";
import { Flyout } from "../ui/flyout/Flyout";
import { useReport } from "../ui/hooks";
import { Root } from "../ui/Root";
import { mount } from "./mount";

function FlyoutWindow() {
  const report = useReport();
  return <Root>{(settings) => <Flyout report={report} settings={settings} />}</Root>;
}

mount(<FlyoutWindow />);
