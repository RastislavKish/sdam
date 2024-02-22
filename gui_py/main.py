#!/usr/bin/env python3

import sys

import toga

from toga import App, MainWindow, Window
from toga import Group, Command
from toga import Box, MultilineTextInput
from toga import Key

import gui_py

class SdamWindow(MainWindow):

    def __init__(self):
        super().__init__("main window", "Untitled - SDAM")

        self._rate=1.0

        recording_group=Group("Recording")
        playback_group=Group("Playback")
        rate_group=Group("Rate", parent=playback_group, order=2)
        forward_group=Group("Forward", parent=playback_group, order=3)
        backward_group=Group("Backward", parent=playback_group, order=4)

        load=Command(self.load,
            text="Load",
            group=Group.APP,
            )
        save=Command(self.save,
            text="Save",
            shortcut=Key.MOD_1+Key.S,
            group=Group.APP,
            )

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

        playback_toggle=Command(self.playback_toggle,
            text="Play / Pause",
            shortcut=Key.MOD_1+Key.K,
            group=playback_group,
            order=1,
            )

        playback_rate_original=Command(self.playback_rate_original,
            text="Original",
            shortcut=Key.MOD_1+Key.I,
            group=rate_group,
            order=1,
            )
        playback_rate_increase=Command(self.playback_rate_increase,
            text="Increase",
            shortcut=Key.MOD_1+Key.SHIFT+Key.O,
            group=rate_group,
            order=2,
            )
        playback_rate_decrease=Command(self.playback_rate_decrease,
            text="Decrease",
            shortcut=Key.MOD_1+Key.SHIFT+Key.U,
            group=rate_group,
            order=3,
            )
        playback_rate_double=Command(self.playback_rate_double,
            text="Double",
            shortcut=Key.MOD_1+Key.O,
            group=rate_group,
            order=4,
            )
        playback_rate_triple=Command(self.playback_rate_triple,
            text="Triple",
            shortcut=Key.MOD_1+Key.P,
            group=rate_group,
            order=5,
            )
        playback_rate_half=Command(self.playback_rate_half,
            text="Half",
            shortcut=Key.MOD_1+Key.U,
            group=rate_group,
            order=6,
            )

        playback_forward_5_seconds=Command(self.playback_forward_5_seconds,
            text="5 seconds",
            shortcut=Key.MOD_1+Key.L,
            group=forward_group,
            order=1,
            )
        playback_forward_10_seconds=Command(self.playback_forward_10_seconds,
            text="10 seconds",
            shortcut=Key.MOD_1+Key.SHIFT+Key.L,
            group=forward_group,
            order=2,
            )
        playback_forward_1_minute=Command(self.playback_forward_1_minute,
            text="1 minute",
            shortcut=Key.MOD_1+Key.SEMICOLON,
            group=forward_group,
            order=3,
            )

        playback_backward_5_seconds=Command(self.playback_backward_5_seconds,
            text="5 seconds",
            shortcut=Key.MOD_1+Key.J,
            group=backward_group,
            order=1,
            )
        playback_backward_10_seconds=Command(self.playback_backward_10_seconds,
            text="10 seconds",
            shortcut=Key.MOD_1+Key.SHIFT+Key.J,
            group=backward_group,
            order=2,
            )
        playback_backward_1_minute=Command(self.playback_backward_1_minute,
            text="1 minute",
            shortcut=Key.MOD_1+Key.H,
            group=backward_group,
            order=3,
            )

        self.toolbar.add(load,
            save,
            recording_start,
            recording_stop,
            playback_toggle,
            playback_rate_original,
            playback_rate_increase,
            playback_rate_decrease,
            playback_rate_double,
            playback_rate_triple,
            playback_rate_half,
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

    async def load(self, sender):
        result=await self.open_file_dialog("LOad a file", file_types=["sdam"], multiple_select=False)

        if result is not None:
            self.load_from_file(str(result))
    async def save(self, sender):
        file_path=gui_py.file_path()

        if file_path=="":
            result=await self.save_file_dialog("Save to file", ".sdam", file_types=["sdam"])

            if result is not None:
                self.save_to_file(str(result))
        else:
            self.save_to_file(file_path)

    def recording_start(self, sender):
        print("Starting recording...")
        gui_py.start_recording()
    def recording_stop(self, sender):
        gui_py.stop_recording()

    def playback_toggle(self, sender):
        gui_py.toggle_playback()

    def playback_rate_original(self, sender):
        self._rate=1.0

        gui_py.set_rate(self._rate)
    def playback_rate_increase(self, sender):
        self._rate+=0.25
        if self._rate>3.0:
            self._rate=3.0

        gui_py.set_rate(self._rate)
    def playback_rate_decrease(self, sender):
        self._rate-=0.25
        if self._rate<=0.0:
            self._rate=0.25

        gui_py.set_rate(self._rate)
    def playback_rate_double(self, sender):
        gui_py.set_rate(2.0)
    def playback_rate_triple(self, sender):
        gui_py.set_rate(3.0)
    def playback_rate_half(self, sender):
        gui_py.set_rate(0.5)

    def playback_forward_5_seconds(self, sender):
        gui_py.forward(5)
    def playback_forward_10_seconds(self, sender):
        gui_py.forward(10)
    def playback_forward_1_minute(self, sender):
        gui_py.forward(60)
    def playback_backward_5_seconds(self, sender):
        gui_py.backward(5)
    def playback_backward_10_seconds(self, sender):
        gui_py.backward(10)
    def playback_backward_1_minute(self, sender):
        gui_py.backward(60)

    def load_from_file(self, path):
        result=gui_py.load(path)

        if result!="":
            self.error_dialog("Error", result)
            return

        self._text_input.value=gui_py.user_text()

        self.title=f"{gui_py.file_name()} - SDAM"
    def save_to_file(self, path):
        gui_py.set_user_text(self._text_input.value)
        gui_py.save(path)
        self.title=f"{gui_py.file_name()} - SDAM"

class SdamApp(App):

    def __init__(self):
        super().__init__("SDAM", "com.rastislavkish.sdam")

    def startup(self):
        sdam_window=SdamWindow()

        self.main_window=sdam_window
        self.main_window.show()

        if len(sys.argv)==2:
            sdam_window.load_from_file(sys.argv[1])

if __name__=="__main__":
    app=SdamApp()
    app.main_loop()
