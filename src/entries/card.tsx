import "../styles/flyout.css";
import "../styles/card.css";
import { Card } from "../ui/card/Card";
import { useReports } from "../ui/hooks";
import { Root } from "../ui/Root";
import { mount } from "./mount";

function CardWindow() {
  const reports = useReports();
  return (
    <Root title="windowTitleCard">
      {(settings) => <Card reports={reports} settings={settings} />}
    </Root>
  );
}

mount(<CardWindow />);
