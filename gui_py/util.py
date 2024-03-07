#!/usr/bin/env python3

import asyncio
from asyncio import Queue

import toga
from toga import Box, Button, Label, TextInput, Window
from toga.style.pack import COLUMN, LEFT, RIGHT, ROW, Pack

def frame_offset_to_time(frame_offset):
    frame_duration=40 #ms
    frames_in_minute=60000//frame_duration
    frames_in_second=1000//frame_duration

    minute=frame_offset//frames_in_minute
    second=(frame_offset%frames_in_minute)//frames_in_second

    return f"{minute:0>2}:{second:0>2}"
async def input_dialog(title, text):
    return await _InputDialog.show_for_result(title, text)

class Toaster:

    def __init__(self, label):
        self._label=label

    def toast(self, text):
        self._label.text=text

class _InputDialog(Window):

    def __init__(self, title, message, result_queue):
        super().__init__(None, title, on_close=self.dialog_close_handler)

        self._result_queue=result_queue
        self._submitted=False

        box=Box()

        input_box=Box()
        self._text_input=TextInput(on_confirm=self.text_input_confirmation_handler)
        input_box.add(self._text_input)
        input_box.add(Button("Ok", on_press=self.ok_button_click_handler))
        box.add(Label(message))
        box.add(input_box)
        box.style.update(direction=COLUMN, padding=10)

        self.content=box

    async def text_input_confirmation_handler(self, sender):
        self._submitted=True
        entered_text=self._text_input.value
        self.close()

        if self._result_queue is not None:
            await self._result_queue.put(entered_text)
    async def ok_button_click_handler(self, sender):
        self._submitted=True
        entered_text=self._text_input.value
        self.close()

        if self._result_queue is not None:
            await self._result_queue.put(entered_text)

    async def dialog_close_handler(self, sender):
        if not self._submitted:
            await self._result_queue.put(None)
        return True

    async def show_for_result(title, text):
        result_queue=Queue()
        dialog=_InputDialog(title, text, result_queue)
        dialog.show()
        result=await result_queue.get()

        return result
