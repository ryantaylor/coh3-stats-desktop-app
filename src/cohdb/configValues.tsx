import { configValueFactory } from "../config-store/configValueFactory"
import {OAuth2Token} from "@badgateway/oauth2-client";

const [getCohdbToken, useCohdbToken] = configValueFactory<OAuth2Token | undefined>(
    "cohdb",
    async () => undefined
)

export { getCohdbToken, useCohdbToken }
