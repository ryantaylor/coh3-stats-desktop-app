import { invoke } from "@tauri-apps/api/tauri"
import app from "@tauri-apps/api/app"

let clientId: string

export const getClientId = async () => {
    if (clientId === undefined) {
        clientId = (await invoke("get_machine_id")) as string
    }
    return clientId
}

let version: string

export const getVersion = async () => {
    if (version === undefined) {
        version = await app.getVersion()
    }
    return version
}
