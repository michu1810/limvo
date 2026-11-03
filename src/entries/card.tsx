import "../styles/flyout.css";
import "../styles/card.css";
import { Card } from "../ui/card/Card";
import { useReport } from "../ui/hooks";
import { Root } from "../ui/Root";
import { mount } from "./mount";

function CardWindow() {
  const report = useReport();
  return <Root>{(settings) => <Card report={report} settings={settings} />}</Root>;
}

mount(<CardWindow />);
