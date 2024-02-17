#!/usr/bin/env python3

import toga

from toga import App, MainWindow, Window
from toga import Group, Command
from toga import Box, MultilineTextInput
from toga import Key

import gui_py

class SdamWindow(MainWindow):

    def __init__(self):
        super().__init__("main window", "Untitled - SDAM")

        recording_group=Group("Recording")
        playback_group=Group("Playback")

        recording_start=Command(self.recording_start,
            text="Start",
            shortcut=Key.MOD_1+Key.R,
            group=recording_group,
            )
        recording_stop=Command(self.recording_stop,
            text="Stop",
            shortcut=Key.MOD_1+Key.SHIFT+Key.R,
            group=recording_group,
            )

        playback_start=Command(self.playback_start,
            text="Start / pause",
            shortcut=Key.MOD_1+Key.K,
            group=playback_group,
            order=1,
            )
        playback_increase_rate=Command(self.playback_increase_rate,
            text="Increase rate",
            shortcut=Key.MOD_1+Key.O,
            group=playback_group,
            order=2,
            )
        playback_decrease_rate=Command(self.playback_decrease_rate,
            text="Decrease rate",
            shortcut=Key.MOD_1+Key.U,
            group=playback_group,
            order=3,
            )
        playback_original_rate=Command(self.playback_original_rate,
            text="Original rate",
            shortcut=Key.MOD_1+Key.I,
            group=playback_group,
            order=4,
            )
        playback_forward_5_seconds=Command(self.playback_forward_5_seconds,
            text="Forward 5 seconds",
            shortcut=Key.MOD_1+Key.L,
            group=playback_group,
            order=5,
            )
        playback_forward_10_seconds=Command(self.playback_forward_10_seconds,
            text="Forward 10 seconds",
            group=playback_group,
            order=6,
            )
        playback_forward_1_minute=Command(self.playback_forward_1_minute,
            text="Forward 1 minute",
            group=playback_group,
            order=7,
            )

        playback_backward_5_seconds=Command(self.playback_backward_5_seconds,
            text="Backward 5 seconds",
            shortcut=Key.MOD_1+Key.J,
            group=playback_group,
            order=8,
            )
        playback_backward_10_seconds=Command(self.playback_backward_10_seconds,
            text="Backward 10 seconds",
            group=playback_group,
            order=9,
            )
        playback_backward_1_minute=Command(self.playback_backward_1_minute,
            text="Backward 1 minute",
            group=playback_group,
            order=10,
            )

        self.toolbar.add(recording_start,
            recording_stop,
            playback_start,
            playback_increase_rate,
            playback_decrease_rate,
            playback_original_rate,
            playback_forward_5_seconds,
            playback_forward_10_seconds,
            playback_forward_1_minute,
            playback_backward_5_seconds,
            playback_backward_10_seconds,
            playback_backward_1_minute,
            )
        self._text_input=MultilineTextInput()
        self.content=self._text_input

        self._text_input.focus()

    def recording_start(self, sender):
        print("Starting recording...")
        gui_py.start_recording()
    def recording_stop(self, sender):
        gui_py.stop_recording()

    def playback_start(self, sender):
        gui_py.start_playback()
    def playback_increase_rate(self, sender):
        pass
    def playback_decrease_rate(self, sender):
        pass
    def playback_original_rate(self, sender):
        pass
    def playback_forward_5_seconds(self, sender):
        gui_py.forward()
    def playback_forward_10_seconds(self, sender):
        pass
    def playback_forward_1_minute(self, sender):
        pass
    def playback_backward_5_seconds(self, sender):
        gui_py.backward()
    def playback_backward_10_seconds(self, sender):
        pass
    def playback_backward_1_minute(self, sender):
        pass

class SdamApp(App):

    def __init__(self):
        super().__init__("SDAM", "com.rastislavkish.sdam")

    def startup(self):
        self.main_window=SdamWindow()
        self.main_window.show()

if __name__=="__main__":
    app=SdamApp()
    app.main_loop()
