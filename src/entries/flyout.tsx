import "../styles/flyout.css";
import { Flyout } from "../ui/flyout/Flyout";
import { useReport } from "../ui/hooks";
import { Root } from "../ui/Root";
import { mount } from "./mount";

function FlyoutWindow() {
  const report = useReport();
  return <Root>{(chosen) => <Flyout report={report} chosenLocale={chosen} />}</Root>;
}

mount(<FlyoutWindow />);
