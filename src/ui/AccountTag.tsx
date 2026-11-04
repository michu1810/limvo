import { accountTag } from "../core/accounts";

/** The small letter beside a service's mark that tells its extra account apart. */
export function AccountTag({ label }: { label: string | null }) {
  if (!label) return null;
  const tag = accountTag(label);
  if (!tag) return null;
  return (
    <span className="account-tag" title={label} aria-hidden="true">
      {tag}
    </span>
  );
}
