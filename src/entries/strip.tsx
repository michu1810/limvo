import "../styles/strip.css";
import { useReports } from "../ui/hooks";
import { Root } from "../ui/Root";
import { Strip } from "../ui/strip/Strip";
import { mount } from "./mount";

function StripWindow() {
  const reports = useReports();
  return (
    <Root title="windowTitleStrip">
      {(settings) => <Strip reports={reports} settings={settings} />}
    </Root>
  );
}

mount(<StripWindow />, { followTaskbarTheme: true });
