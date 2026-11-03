import "../styles/strip.css";
import { useReport } from "../ui/hooks";
import { Root } from "../ui/Root";
import { Strip } from "../ui/strip/Strip";
import { mount } from "./mount";

function StripWindow() {
  const report = useReport();
  return <Root>{(settings) => <Strip report={report} settings={settings} />}</Root>;
}

mount(<StripWindow />, { followTaskbarTheme: true });
