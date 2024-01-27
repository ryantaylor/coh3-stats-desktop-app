import React from "react"
import ReactDOM from "react-dom/client"
import { Providers } from "./Providers"
import { Router } from "./Router"
import { renderStreamerHTML } from "./streamer-overlay/renderStreamerOverlay"
import events from "./mixpanel/mixpanel"
import { listen } from "@tauri-apps/api/event"
import { invoke } from '@tauri-apps/api';
import { appWindow } from "@tauri-apps/api/window"
import { trace, info, error, attachConsole } from "tauri-plugin-log-api"
import {useCohdbToken} from "./cohdb/configValues";
import {cohdbWrapper, tokenFromRedirect} from "./cohdb/oauth";
import {getStore} from "./config-store/store";
import {CONFIG_CHANGE_EVENT} from "./config-store/configValueFactory";

info("Start frontend")

events.init()

// make sure an html file exists
renderStreamerHTML({
  uniqueID: "",
  state: "Closed",
  type: "Classic",
  timestamp: "",
  duration: 0,
  map: "",
  winCondition: "",
  left: {
    players: [],
    side: "Mixed",
  },
  right: {
    players: [],
    side: "Mixed",
  },
  language_code: "",
})

listen("single-instance", () => {
  //appWindow.requestUserAttention(2)
  //appWindow.setFocus()
})

// Wait for callback from tauri oauth plugin
listen('oauth://url', async (data) => {
  const token = await tokenFromRedirect(data.payload as string);
  const store = await getStore();
  await store.set('cohdb', token);
  await store.save();
  CONFIG_CHANGE_EVENT.emit('cohdb', token)
});

// invoke('start_server').then(port => console.log(port))

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Providers>
      <Router />
    </Providers>
  </React.StrictMode>
)
