#!/usr/bin/env python3

import asyncio
from asyncio import Queue
import re
import sys

import toga

from toga import App, MainWindow, Window
from toga import Group, Command
from toga import Box, Button, Label, MultilineTextInput, Table, TextInput
from toga.style.pack import COLUMN, LEFT, RIGHT, ROW, Pack
from toga import Key

import gui_py
from gui_py import PyMark

import util
from util import frame_offset_to_time, input_dialog, Toaster

TIME_REGEX=re.compile(r"^(\d*:){,2}\d+$")

class ViewMarksWindow(Window):

    def __init__(self, result_queue=None):
        super().__init__(None, "Marks", on_close=self.window_close_handler)

        self._submitted=False
        self._result_queue=result_queue

        table=Table(
            headings=["Mark", "Category", "Time"],
            data=[],
            on_activate=self.table_activate_handler,
            )
        self._table=table

        control_bar=Box()
        control_bar.add(Button("Jump to", on_press=self.jump_to_button_press_handler))
        control_bar.add(Button("Set label", on_press=self.set_label_button_press_handler))
        control_bar.add(Button("Delete", on_press=self.delete_button_press_handler))

        content=Box()
        content.add(table)
        content.add(control_bar)
        content.style.update(direction=COLUMN, padding=10)

        self.content=content

        self.populate_table()

    async def jump_to_button_press_handler(self, sender):
        mark=self.get_selected_mark()

        if mark is None:
            return

        await self.jump_to_mark(mark)
    async def set_label_button_press_handler(self, sender):
        mark=self.get_selected_mark()

        if mark is None:
            return

        new_label=await input_dialog("Set label", f"Set label of mark \"{mark.label}\" to:")

        if new_label is None:
            return
        if new_label=="":
            new_label=None

        mark.label=new_label
        gui_py.edit_mark(mark.id, mark)

        self.populate_table()
    def delete_button_press_handler(self, sender):
        mark=self.get_selected_mark()

        if mark is None:
            return

        gui_py.delete_mark(mark.id)

        self.populate_table()

    async def table_activate_handler(self, sender, **row):
        if not "row" in row:
            return

        index=self._table.data.index(row["row"])
        mark=self._marks[index]

        await self.jump_to_mark(mark)
    async def window_close_handler(self, sender):
        if not self._submitted and self._result_queue is not None:
            await self._result_queue.put(None)
        return True

    def populate_table(self):
        self._marks=gui_py.marks()
        self._marks.sort(key=lambda mark: mark.frame_offset)

        table_data=[]
        for mark in self._marks:
            data_entry=(mark.label, mark.category, frame_offset_to_time(mark.frame_offset))
            table_data.append(data_entry)

        self._table.data=table_data
    async def jump_to_mark(self, mark):
        if self._result_queue is not None:
            self._submitted=True
            self.close()
            await self._result_queue.put(mark)

    def get_selected_index(self):
        selection=self._table.selection

        if selection is None:
            return None

        index=self._table.data.index(selection)

        return index
    def get_selected_mark(self):
        index=self.get_selected_index()

        if index is None:
            return None

        mark=self._marks[index]

        return mark

    async def show_for_result():
        result_queue=Queue()

        window=ViewMarksWindow(result_queue)
        window.show()

        result=await result_queue.get()

        return result

class SdamWindow(MainWindow):

    def __init__(self):
        super().__init__("main window", "Untitled - SDAM")

        self._rate=1.0
        self._time_travel=False
        self._recording_before_timetravel=False
        self._focused_mark=None

        recording_group=Group("Recording")
        playback_group=Group("Playback")
        rate_group=Group("Rate",
            parent=playback_group,
            order=2,
            )
        forward_group=Group("Forward",
            parent=playback_group,
            order=3,
            )
        backward_group=Group("Backward",
            parent=playback_group,
            order=4,
            )
        jump_to_group=Group("Jump to",
            parent=playback_group,
            order=5,
            )
        percentage_group=Group("Percentage",
            parent=jump_to_group,
            order=3,
            )
        time_travel_group=Group("Time travel")
        marks_group=Group("Marks")
        marks_add_group=Group("Add", parent=marks_group, order=1)
        marks_add_labeled_group=Group("Add labeled", parent=marks_group, order=2)
        marks_jump_to_group=Group("Jump to", parent=marks_group, order=3)
        marks_edit_focused_mark_group=Group("Edit focused mark", parent=marks_group, order=4)


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

        playback_jump_to_start=Command(self.playback_jump_to_start,
            text="Start",
            group=jump_to_group,
            order=1,
            )
        playback_jump_to_end=Command(self.playback_jump_to_end,
            text="End",
            group=jump_to_group,
            order=2,
            )
        playback_jump_to_percentage_0=Command(self.playback_jump_to_percentage_0,
            text="0%",
            shortcut=Key.MOD_1+"0",
            group=percentage_group,
            order=1,
            )
        playback_jump_to_percentage_10=Command(self.playback_jump_to_percentage_10,
            text="10%",
            shortcut=Key.MOD_1+"1",
            group=percentage_group,
            order=2,
            )
        playback_jump_to_percentage_20=Command(self.playback_jump_to_percentage_20,
            text="20%",
            shortcut=Key.MOD_1+"2",
            group=percentage_group,
            order=3,
            )
        playback_jump_to_percentage_30=Command(self.playback_jump_to_percentage_30,
            text="30%",
            shortcut=Key.MOD_1+"3",
            group=percentage_group,
            order=4,
            )
        playback_jump_to_percentage_40=Command(self.playback_jump_to_percentage_40,
            text="40%",
            shortcut=Key.MOD_1+"4",
            group=percentage_group,
            order=5,
            )
        playback_jump_to_percentage_50=Command(self.playback_jump_to_percentage_50,
            text="50%",
            shortcut=Key.MOD_1+"5",
            group=percentage_group,
            order=6,
            )
        playback_jump_to_percentage_60=Command(self.playback_jump_to_percentage_60,
            text="60%",
            shortcut=Key.MOD_1+"6",
            group=percentage_group,
            order=7,
            )
        playback_jump_to_percentage_70=Command(self.playback_jump_to_percentage_70,
            text="70%",
            shortcut=Key.MOD_1+"7",
            group=percentage_group,
            order=8,
            )
        playback_jump_to_percentage_80=Command(self.playback_jump_to_percentage_80,
            text="80%",
            shortcut=Key.MOD_1+"8",
            group=percentage_group,
            order=9,
            )
        playback_jump_to_percentage_90=Command(self.playback_jump_to_percentage_90,
            text="90%",
            shortcut=Key.MOD_1+"9",
            group=percentage_group,
            order=10,
            )
        playback_jump_to_percentage_100=Command(self.playback_jump_to_percentage_100,
            text="100%",
            group=percentage_group,
            order=11,
            )
        playback_jump_to_time=Command(self.playback_jump_to_time,
            text="Time",
            shortcut=Key.MOD_1+Key.MOD_2+Key.T,
            group=jump_to_group,
            order=4,
            )

        time_travel_activate=Command(self.time_travel_activate,
            text="Activate",
            shortcut=Key.MOD_1+Key.T,
            group=time_travel_group,
            )
        time_travel_deactivate=Command(self.time_travel_deactivate,
            text="Deactivate",
            shortcut=Key.MOD_1+Key.SHIFT+Key.T,
            group=time_travel_group,
            )

        marks_add_category_1_mark=Command(self.marks_add_category_1_mark,
            text="Category 1 mark",
            shortcut=Key.MOD_2+"1",
            group=marks_add_group,
            )
        marks_add_category_2_mark=Command(self.marks_add_category_2_mark,
            text="Category 2 mark",
            shortcut=Key.MOD_2+"2",
            group=marks_add_group,
            )
        marks_add_category_3_mark=Command(self.marks_add_category_3_mark,
            text="Category 3 mark",
            shortcut=Key.MOD_2+"3",
            group=marks_add_group,
            )
        marks_add_category_4_mark=Command(self.marks_add_category_4_mark,
            text="Category 4 mark",
            shortcut=Key.MOD_2+"4",
            group=marks_add_group,
            )
        marks_add_category_5_mark=Command(self.marks_add_category_5_mark,
            text="Category 5 mark",
            shortcut=Key.MOD_2+"5",
            group=marks_add_group,
            )

        marks_add_labeled_category_1_mark=Command(self.marks_add_labeled_category_1_mark,
            text="Category 1 mark",
            shortcut=Key.MOD_1+Key.MOD_2+"1",
            group=marks_add_labeled_group,
            )
        marks_add_labeled_category_2_mark=Command(self.marks_add_labeled_category_2_mark,
            text="Category 2 mark",
            shortcut=Key.MOD_1+Key.MOD_2+"2",
            group=marks_add_labeled_group,
            )
        marks_add_labeled_category_3_mark=Command(self.marks_add_labeled_category_3_mark,
            text="Category 3 mark",
            shortcut=Key.MOD_1+Key.MOD_2+"3",
            group=marks_add_labeled_group,
            )
        marks_add_labeled_category_4_mark=Command(self.marks_add_labeled_category_4_mark,
            text="Category 4 mark",
            shortcut=Key.MOD_1+Key.MOD_2+"4",
            group=marks_add_labeled_group,
            )
        marks_add_labeled_category_5_mark=Command(self.marks_add_labeled_category_5_mark,
            text="Category 5 mark",
            shortcut=Key.MOD_1+Key.MOD_2+"5",
            group=marks_add_labeled_group,
            )

        marks_jump_to_next_mark=Command(self.marks_jump_to_next_mark,
            text="Next mark",
            shortcut=Key.MOD_1+".",
            order=1,
            group=marks_jump_to_group,
            )
        marks_jump_to_next_closest_mark=Command(self.marks_jump_to_next_closest_mark,
            text="Next closest mark",
            shortcut=Key.MOD_1+Key.SHIFT+".",
            order=2,
            group=marks_jump_to_group,
            )
        marks_jump_to_previous_mark=Command(self.marks_jump_to_previous_mark,
            text="Previous mark",
            shortcut=Key.MOD_1+Key.M,
            order=10,
            group=marks_jump_to_group,
            )
        marks_jump_to_previous_closest_mark=Command(self.marks_jump_to_previous_closest_mark,
            text="Previous closest mark",
            shortcut=Key.MOD_1+Key.SHIFT+Key.M,
            order=11,
            group=marks_jump_to_group,
            )
        marks_jump_to_focused_mark=Command(self.marks_jump_to_focused_mark,
            text="Focused mark",
            shortcut=Key.MOD_1+Key.COMMA,
            order=20,
            group=marks_jump_to_group,
            )

        marks_edit_focused_mark_label=Command(self.marks_edit_focused_mark_label,
            text="Label",
            shortcut=Key.MOD_1+Key.SHIFT+Key.COMMA,
            order=1,
            group=marks_edit_focused_mark_group,
            )
        marks_edit_focused_mark_move_to_current_position=Command(self.marks_edit_focused_mark_move_to_current_position,
            text="Move to current position",
            shortcut=Key.MOD_1+Key.MOD_2+Key.COMMA,
            order=2,
            group=marks_edit_focused_mark_group,
            )
        marks_edit_focused_mark_delete=Command(self.marks_edit_focused_mark_delete,
            text="Delete",
            shortcut=Key.MOD_1+Key.MOD_2+Key.SHIFT+Key.COMMA,
            order=3,
            group=marks_edit_focused_mark_group,
            )

        marks_view=Command(self.marks_view,
            text="View",
            order=5,
            group=marks_group,
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
            playback_jump_to_start,
            playback_jump_to_end,
            playback_jump_to_percentage_0,
            playback_jump_to_percentage_10,
            playback_jump_to_percentage_20,
            playback_jump_to_percentage_30,
            playback_jump_to_percentage_40,
            playback_jump_to_percentage_50,
            playback_jump_to_percentage_60,
            playback_jump_to_percentage_70,
            playback_jump_to_percentage_80,
            playback_jump_to_percentage_90,
            playback_jump_to_percentage_100,
            playback_jump_to_time,
            time_travel_activate,
            time_travel_deactivate,
            marks_add_category_1_mark,
            marks_add_category_2_mark,
            marks_add_category_3_mark,
            marks_add_category_4_mark,
            marks_add_category_5_mark,
            marks_add_labeled_category_1_mark,
            marks_add_labeled_category_2_mark,
            marks_add_labeled_category_3_mark,
            marks_add_labeled_category_4_mark,
            marks_add_labeled_category_5_mark,
            marks_jump_to_next_mark,
            marks_jump_to_next_closest_mark,
            marks_jump_to_previous_mark,
            marks_jump_to_previous_closest_mark,
            marks_jump_to_focused_mark,
            marks_edit_focused_mark_label,
            marks_edit_focused_mark_move_to_current_position,
            marks_edit_focused_mark_delete,
            marks_view,
            )

        content=Box()

        text_input=MultilineTextInput()
        self._text_input=text_input

        status_bar=Box()
        current_position_label=Label("")
        self._current_position_label=current_position_label
        toast_label=Label("")
        self._toaster=Toaster(toast_label)

        status_bar.add(current_position_label)
        status_bar.add(toast_label)

        content.add(text_input)
        content.add(status_bar)

        content.style.update(direction=COLUMN, padding=10)

        self.content=content

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
        if not gui_py.is_recording():
            print("Starting recording...")
            gui_py.pause_playback()
            gui_py.start_recording()

            if gui_py.is_recording():
                self._toaster.toast("Recording")
    def recording_stop(self, sender):
        if not self._time_travel:
            gui_py.stop_recording()
            if not gui_py.is_recording():
                self._toaster.toast("Recording stopped")
        else:
            self._recording_before_time_travel=False
            self.time_travel_deactivate(None)

            if not gui_py.is_recording():
                self._toaster.toast("Recording stopped, Timetravel stopped")

    def playback_toggle(self, sender):
        if gui_py.is_recording() and not self._time_travel and not gui_py.is_playing():
            return
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

    def playback_jump_to_start(self, sender):
        gui_py.jump_to_start()
    def playback_jump_to_end(self, sender):
        gui_py.jump_to_end()
    def playback_jump_to_percentage_0(self, sender):
        self.playback_jump_to_percentage(0)
    def playback_jump_to_percentage_10(self, sender):
        self.playback_jump_to_percentage(10)
    def playback_jump_to_percentage_20(self, sender):
        self.playback_jump_to_percentage(20)
    def playback_jump_to_percentage_30(self, sender):
        self.playback_jump_to_percentage(30)
    def playback_jump_to_percentage_40(self, sender):
        self.playback_jump_to_percentage(40)
    def playback_jump_to_percentage_50(self, sender):
        self.playback_jump_to_percentage(50)
    def playback_jump_to_percentage_60(self, sender):
        self.playback_jump_to_percentage(60)
    def playback_jump_to_percentage_70(self, sender):
        self.playback_jump_to_percentage(70)
    def playback_jump_to_percentage_80(self, sender):
        self.playback_jump_to_percentage(80)
    def playback_jump_to_percentage_90(self, sender):
        self.playback_jump_to_percentage(90)
    def playback_jump_to_percentage_100(self, sender):
        self.playback_jump_to_percentage(100)

    def playback_jump_to_percentage(self, percentage):
        gui_py.jump_to_percentage(percentage)

    async def playback_jump_to_time(self, sender):
        text=await input_dialog("Jump to time", "Enter the time to jump to, in minute, minute:second or hour:minute:second format.")

        if text is None or text=="":
            return

        if not TIME_REGEX.match(text):
            await self.error_dialog("Error", f"'{text}' is not a valid time")
            return

        parts=[int(i) for i in text.split(":")]

        seconds=0

        if len(parts)==1:
            seconds=60*parts[0]
        elif len(parts)==2:
            seconds=60*parts[0]+parts[1]
        elif len(parts)==3:
            seconds=3600*parts[0]+60*parts[1]+parts[2]

        gui_py.jump_to_time(seconds)

    def time_travel_activate(self, sender):
        if not self._time_travel:
            recording=gui_py.is_recording()

            gui_py.pause_playback()
            if not recording:
                gui_py.start_recording()
            gui_py.jump_to_end()
            gui_py.start_playback()

            self._recording_before_time_travel=recording
            self._time_travel=True

            self._toaster.toast("Timetravel activated")
    def time_travel_deactivate(self, sender):
        if self._time_travel:
            gui_py.pause_playback()
            if not self._recording_before_time_travel:
                gui_py.stop_recording()
            self._time_travel=False

            self._toaster.toast("Timetravel deactivated")

    def marks_add_category_1_mark(self, sender):
        self.marks_add_mark(1, None)
    def marks_add_category_2_mark(self, sender):
        self.marks_add_mark(2, None)
    def marks_add_category_3_mark(self, sender):
        self.marks_add_mark(3, None)
    def marks_add_category_4_mark(self, sender):
        self.marks_add_mark(4, None)
    def marks_add_category_5_mark(self, sender):
        self.marks_add_mark(5, None)

    async def marks_add_labeled_category_1_mark(self, sender):
        await self.marks_add_labeled_mark(1)
    async def marks_add_labeled_category_2_mark(self, sender):
        await self.marks_add_labeled_mark(2)
    async def marks_add_labeled_category_3_mark(self, sender):
        await self.marks_add_labeled_mark(3)
    async def marks_add_labeled_category_4_mark(self, sender):
        await self.marks_add_labeled_mark(4)
    async def marks_add_labeled_category_5_mark(self, sender):
        await self.marks_add_labeled_mark(5)
    async def marks_add_labeled_mark(self, category):
        label=await input_dialog("Add mark", f"Add a category {category} mark with label:")

        if label is not None:
            self.marks_add_mark(category, label)

    def marks_add_mark(self, category, label):
        position=None

        if self._time_travel:
            position=gui_py.current_position()
        elif gui_py.is_recording():
            position=gui_py.audio_len()
        else:
            position=gui_py.current_position()

        if position is None:
            return

        gui_py.add_mark(PyMark(position, category, label))

        self._toaster.toast("Mark added")

    def marks_jump_to_next_mark(self, sender):
        if self._focused_mark is not None:
            next_mark=gui_py.next_closest_mark(self._focused_mark.frame_offset)
            if next_mark is not None:
                self._focused_mark=next_mark
                gui_py.jump_to_frame(next_mark.frame_offset)
        else:
            self.marks_jump_to_next_closest_mark(None)
    def marks_jump_to_next_closest_mark(self, sender):
        current_position=gui_py.current_position()

        if current_position is None:
            current_position=0

        next_closest_mark=gui_py.next_closest_mark(current_position)

        if next_closest_mark is not None:
            self._focused_mark=next_closest_mark
            gui_py.jump_to_frame(next_closest_mark.frame_offset)
    def marks_jump_to_previous_mark(self, sender):
        if self._focused_mark is not None:
            previous_mark=gui_py.previous_closest_mark(self._focused_mark.frame_offset)

            if previous_mark is not None:
                self._focused_mark=previous_mark
                gui_py.jump_to_frame(previous_mark.frame_offset)
        else:
            self.marks_jump_to_previous_closest_mark(None)
    def marks_jump_to_previous_closest_mark(self, sender):
        current_position=gui_py.current_position()

        if current_position is None:
            return

        previous_closest_mark=gui_py.previous_closest_mark(current_position)

        if previous_closest_mark is not None:
            self._focused_mark=previous_closest_mark
            gui_py.jump_to_frame(previous_closest_mark.frame_offset)

    def marks_jump_to_focused_mark(self, sender):
        if self._focused_mark is not None:
            gui_py.jump_to_frame(self._focused_mark.frame_offset)

    async def marks_edit_focused_mark_label(self, sender):
        if self._focused_mark is not None:
            original_label=self._focused_mark.label
            new_label=await input_dialog("Set label", f"Set label of mark \"{original_label}\" to:")

            if new_label is None:
                return

            if new_label=="":
                new_label=None

            self._focused_mark.label=new_label

            gui_py.edit_mark(self._focused_mark.id, self._focused_mark)
    def marks_edit_focused_mark_move_to_current_position(self, sender):
        if self._focused_mark is not None:
            current_position=gui_py.current_position()

            if current_position is not None:
                self._focused_mark.frame_offset=current_position
                gui_py.edit_mark(self._focused_mark.id, self._focused_mark)
                self._toaster.toast("Mark moved")
    def marks_edit_focused_mark_delete(self, sender):
        if self._focused_mark is not None:
            gui_py.delete_mark(self._focused_mark.id)
            self._focused_mark=None
            self._toaster.toast("Mark deleted")

    async def marks_view(self, sender):
        mark_to_focus=await ViewMarksWindow.show_for_result()

        if mark_to_focus is not None:
            self._focused_mark=mark_to_focus
            self.marks_jump_to_focused_mark(None)
            return

        # If a mark was focused before, we need to update it in case it was modified
        if self._focused_mark is not None:
            self._focused_mark=gui_py.get_mark(self._focused_mark.id)

    def timer(self):
        current_position=gui_py.current_position()

        if current_position is None:
            current_position=0

        audio_len=gui_py.audio_len()

        self._current_position_label.text=f"{frame_offset_to_time(current_position)} / {frame_offset_to_time(audio_len)}"
    def release(self):
        self._toaster.release()

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
        super().__init__("SDAM", "com.rastislavkish.sdam", on_exit=self.app_exit_handler)

    def startup(self):
        sdam_window=SdamWindow()

        self.main_window=sdam_window
        self.main_window.show()

        if len(sys.argv)==2:
            sdam_window.load_from_file(sys.argv[1])

        self.loop.call_later(1, self.timer_executor)

    def timer_executor(self):
        self.main_window.timer()
        self.loop.call_later(5, self.timer_executor)

    def app_exit_handler(self, sender):
        self.main_window.release()
        return True

if __name__=="__main__":
    app=SdamApp()
    app.main_loop()
