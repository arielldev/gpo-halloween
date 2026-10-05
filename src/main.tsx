import React from "react";
import ReactDOM from "react-dom/client";
import "./index.css";
import Hud from "./windows/Hud";
import Panel from "./windows/Panel";
import Overlay from "./windows/Overlay";
import Guide from "./windows/Guide";

const view = new URLSearchParams(window.location.search).get("view") ?? "panel";

const Root = view === "hud" ? Hud : view === "overlay" ? Overlay : view === "guide" ? Guide : Panel;

document.addEventListener("contextmenu", (e) => e.preventDefault());

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <Root />
  </React.StrictMode>,
);
