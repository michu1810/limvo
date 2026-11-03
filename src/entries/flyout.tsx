import "../styles/flyout.css";
import { Flyout } from "../ui/flyout/Flyout";
import { useReports } from "../ui/hooks";
import { Root } from "../ui/Root";
import { mount } from "./mount";

function FlyoutWindow() {
  const reports = useReports();
  return <Root>{(settings) => <Flyout reports={reports} settings={settings} />}</Root>;
}

mount(<FlyoutWindow />);
