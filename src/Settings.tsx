import { ColorSchemeToggle } from "coh-stats-components"
import {
  Box,
  Group,
  Stack,
  Divider,
  Input,
  ActionIcon,
  Text,
  List,
  Button,
  Tooltip,
  Checkbox,
  Slider,
  Anchor,
} from "@mantine/core"
import { appDataDir } from "@tauri-apps/api/path"
import { writeText } from "@tauri-apps/api/clipboard"
import { useEffect, useState } from "react"
import { IconCheck, IconCopy, IconPlayerPlay, IconX } from "@tabler/icons-react"
import {message, open} from "@tauri-apps/api/dialog"
import { open as openLink } from "@tauri-apps/api/shell"
import {useLogFilePath, usePlaybackPath} from "./game-data-provider/configValues"
import {
  usePlaySound,
  usePlaySoundVolume,
} from "./game-found-sound/configValues"
import {
  useShowFlagsOverlay,
  useAlwaysShowOverlay,
} from "./streamer-overlay/configValues"
import { playSound as playSoundFunc } from "./game-found-sound/playSound"
import events from "./mixpanel/mixpanel"
import { useGameData } from "./game-data-provider/GameDataProvider"
import {invoke} from "@tauri-apps/api/tauri";
import {emit, listen} from "@tauri-apps/api/event";
import {Simulate} from "react-dom/test-utils";
import play = Simulate.play;

interface CohdbUser {
  name: string;
  profile_id: number;
  steam_id: number;
}

export const Settings: React.FC = () => {
  const gameData = useGameData()
  const [logFilePath, setLogFilePath] = useLogFilePath()
  const [playbackPath, setPlaybackPath] = usePlaybackPath()
  const [playSound, setPlaySound] = usePlaySound()
  const [playSoundVolume, setPlaySoundVolume] = usePlaySoundVolume()
  const [showFlagsOverlay, setShowFlagsOverlay] = useShowFlagsOverlay()
  const [alwaysShowOverlay, setAlwaysShowOverlay] = useAlwaysShowOverlay()
  const [appDataPath, setAppDataPath] = useState<string>("")
  const [cohdbUser, setCohdbUser] = useState<CohdbUser | null>(null);

  useEffect(() => {
    events.open_settings()
  }, [])

  useEffect(() => {
    const getAppDataPath = async () => {
      const path = await appDataDir()
      setAppDataPath(path)
    }
    if (appDataPath === "") {
      getAppDataPath()
    }
  }, [appDataPath])

  useEffect(() => {
    const getCohdbUser = async () => {
      const user = await invoke('plugin:cohdb|connected') as CohdbUser | null;
      setCohdbUser(user);
    }

    const unlisten = listen<CohdbUser | null>('cohdb:connection', (event) => {
      getCohdbUser();
      if (event.payload != null) {
        message('Your account has been connected! You can close the browser window now.');
      }
    });

    getCohdbUser();

    return () => {
      unlisten.then(f => f());
    }
  }, [])

  const openLogfileDialog = async () => {
    const selected = await open({
      title: "Select Coh3 warnings.log file",
      multiple: false,
      directory: false,
      defaultPath: logFilePath,
      filters: [
        {
          name: "Logfile",
          extensions: ["log"],
        },
      ],
    })
    if (selected !== null) {
      events.settings_changed("logFilePath", selected as string)
      setLogFilePath(selected as string)
    }
  }

  const openPlaybackDialog = async () => {
    const selected = await open({
      title: "Select CoH3 playback directory",
      multiple: false,
      directory: true,
      defaultPath: playbackPath,
    })
    if (selected !== null) {
      events.settings_changed("playbackPath", selected as string)
      setPlaybackPath(selected as string).then(() => emit('playback-dir-changed', selected))
    }
  }

  return (
    <>
      <Box p="xl">
        <Stack>
          <Group>
            <div>Color Theme:</div>
            <div>
              <ColorSchemeToggle />
            </div>
          </Group>
          <Group>
            <div>Path to warnings.log:</div>
            <div>
              <Group spacing="xs">
                <Group spacing={3}>
                  <Input
                    value={logFilePath ? logFilePath : ""}
                    style={{ width: 500 }}
                    readOnly
                  />
                  <Button variant="default" onClick={openLogfileDialog}>
                    Select
                  </Button>
                </Group>
                <Tooltip
                  label={
                    logFilePath !== undefined
                      ? "Log file found"
                      : "Could not find log file"
                  }
                >
                  <ActionIcon
                    variant="light"
                    color={logFilePath !== undefined ? "green" : "red"}
                    radius="xl"
                  >
                    {logFilePath !== undefined ? (
                      <IconCheck size="1.125rem" />
                    ) : (
                      <IconX size="1.125rem" />
                    )}
                  </ActionIcon>
                </Tooltip>
              </Group>
            </div>
          </Group>
          <Group>
            <div>Play sound on match found:</div>
            <div>
              <Group>
                <Checkbox
                  checked={playSound === undefined ? false : playSound}
                  onChange={(event) => {
                    events.settings_changed(
                      "play_sound",
                      `${event.currentTarget.checked}`
                    )
                    setPlaySound(event.currentTarget.checked)
                  }}
                />
                <Text>Volume:</Text>
                <Slider
                  min={0.1}
                  max={1}
                  step={0.1}
                  style={{ width: "100px" }}
                  label={
                    playSoundVolume ? <>{playSoundVolume.toFixed(1)}</> : null
                  }
                  value={playSoundVolume}
                  onChange={setPlaySoundVolume}
                  onChangeEnd={(value) => {
                    events.settings_changed("play_sound_volume", value)
                  }}
                />
                <Tooltip label="Play sound">
                  <ActionIcon
                    radius="xl"
                    variant="filled"
                    color="blue"
                    onClick={playSoundFunc}
                  >
                    <IconPlayerPlay size="1.125rem" />
                  </ActionIcon>
                </Tooltip>
              </Group>
            </div>
          </Group>
          <Divider />
          <Text weight={700}>OBS Streamer Overlay:</Text>
          <Group>
            <div>Only show stats when loading / ingame:</div>
            <div>
              <Checkbox
                checked={
                  alwaysShowOverlay === undefined ? true : !alwaysShowOverlay
                }
                onChange={(event) => {
                  events.settings_changed(
                    "alwaysShowOverlay",
                    `${!event.currentTarget.checked}`
                  )
                  setAlwaysShowOverlay(!event.currentTarget.checked)
                  if (gameData) {
                    gameData.reloadLogFile()
                  }
                }}
              />
            </div>
          </Group>
          <Group>
            <div>Show flags:</div>
            <div>
              <Checkbox
                checked={
                  showFlagsOverlay === undefined ? false : showFlagsOverlay
                }
                onChange={(event) => {
                  events.settings_changed(
                    "showFlagsOverlay",
                    `${event.currentTarget.checked}`
                  )
                  setShowFlagsOverlay(event.currentTarget.checked)
                  if (gameData) {
                    gameData.reloadLogFile()
                  }
                }}
              />
            </div>
          </Group>
          <div>
            <Text weight={700}>
              Follow the{" "}
              <Anchor
                onClick={() =>
                  openLink(
                    "https://github.com/cohstats/coh3-stats-desktop-app#setup-obs-streamer-overlay"
                  )
                }
              >
                Setup instructions
              </Anchor>{" "}
              &{" "}
              <Anchor
                onClick={() =>
                  openLink(
                    "https://github.com/cohstats/coh3-stats-desktop-app#custom-css-for-the-overlay"
                  )
                }
              >
                Custom CSS instructions
              </Anchor>
            </Text>
          </div>
          <Group>
            <Text>Path to streamerOverlay.html:</Text>
            <Input value={appDataPath} style={{ width: 500 }} readOnly />
            <Tooltip label="Copy">
              <ActionIcon
                onClick={() => {
                  writeText(appDataPath)
                }}
              >
                <IconCopy size="1.125rem" />
              </ActionIcon>
            </Tooltip>
          </Group>
          <Group>
            {cohdbUser != null ? (
              <>
                Connected as {cohdbUser.name}!
                <Button variant="default" onClick={() => invoke('plugin:cohdb|disconnect')}>
                  Disconnect
                </Button>
                <Button variant="filled" onClick={() => invoke('plugin:cohdb|upload')}>
                  Upload
                </Button>
              </>
            ) : (
              <Button variant="default" onClick={() => invoke('plugin:cohdb|authenticate')}>
                Authenticate with cohdb!
              </Button>
            )}
          </Group>
          {cohdbUser != null && <Group>
            <div>Path to playback directory:</div>
            <div>
              <Group spacing="xs">
                <Group spacing={3}>
                  <Input
                    value={playbackPath ? playbackPath : ""}
                    style={{ width: 500 }}
                    readOnly
                  />
                  <Button variant="default" onClick={openPlaybackDialog}>
                    Select
                  </Button>
                </Group>
                <Tooltip
                  label={
                    playbackPath !== undefined
                      ? "Playback directory found"
                      : "Could not find playback directory"
                  }
                >
                  <ActionIcon
                    variant="light"
                    color={playbackPath !== undefined ? "green" : "red"}
                    radius="xl"
                  >
                    {playbackPath !== undefined ? (
                      <IconCheck size="1.125rem" />
                    ) : (
                      <IconX size="1.125rem" />
                    )}
                  </ActionIcon>
                </Tooltip>
              </Group>
            </div>
          </Group>}
        </Stack>
      </Box>
    </>
  )
}
