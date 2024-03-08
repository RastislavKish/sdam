# Semantic Digital Audio Memory

While attending classes of theoretical mathematics, I'm usually facing 3 problems:

1. I can't write down notes and pay attention at the same time
2. Sometimes, I don't get the context of the explained concept right away, I need few moments to think it through or even lookup additional details in my notes or on the Internet. So, I either don't do so and end up just sitting in the class being unable to understand anything, because that concept was important for later topics, or, I do the lookup asynchronously, what however means I get out of sync with the explanation and find myself in the same situation, except now I can't do much with it.
3. If the class requires active work, my mind gets submerged in the problem and can't track anything in the physical world, resulting in shattered context and missed information.

Recording classes can fix all of these issues, however for the cost of doubling the processing time for each class, since raw recordings don't hold any information about their content and need to be listened through in full to get a good-quality notes.

### Semantic audio

SDAM lets you capture recordings with assigned meaning. In the simplest usage, you can just start the recording and add a mark whenever something you will want to write down later is said, when the class is over, you can just return to those labels and quickly create the notes, you can be sure you have covered everything important without the need to go through the whole thing again. At the same time, those marks can serve as reference points, if you need to return in your memory to the part of your class dealing with a particular topic, because you feel you may have missed something or just want to hear it again, you can get to the relevant part in few clicks.

### Time travel

However, SDAM also offers a different operation mode. If you have headphones with active noise cancellation technology, you can use it to travel in time during the class. After activating this function, the program will work in augmented reality mode, where you can hear what's happening around you. And if you don't get something, need to research or simply mishear, there's nothing simpler than pausing the time or rewinding it back, you will get to repeat the past events without missing on anything that's happening in the meantime, because everything is being recorded for you in the background. So when you're done, you can simply continue listening to the class as it was happening while you were dealing with other things, or, even increase the speed twice or triple to get in sync again.

The program is also equipped with a built-in notepad, so you can make use of it to do your note-taking stuff, calculations and other textual operations.

### Saving your memory to a file

When the class is over and you save everything, all the recorded audio, taken marks and written notes is put into a single file, which can be afterwards opened again in SDAM and act as a effective capture of your memory back from the class.

## Installation

The easiest way to get started is to simply grab the precompiled binaries from the [Releases](https://github.com/RastislavKish/sdam/releases) section of the GitHub repository. Currently, Linux and Windows are supported.

## Usage

SDAM is designed to be efficient to use. Even though controlling it subconsciously will likely take some practice, eventually this process should be as seamless as possible, so you can fully focus on the class instead of your note-taking program.

For sake of efficiency, most of the actions are supposed to be invoked from keyboard. The block of keys U I O, J K L and M , ., together with various modifiers, let you control the rate, playback and currently focused mark, respectively. The Number row allows you to quickly place marks (labelled or unlabelled) in combination with Alt, while pressing them with Ctrl will seek to n-th tenth of the recording.

There are also shortcuts for other operations, like jumping to a specific time, saving your changes or controlling the recording, see the program menu for more details.

## Build from source

You can also build the project from source, although the process requires some setup.

### Dependencies

In order to build the project, you need to have the following:

* [Rust programming language](https://www.rust-lang.org/tools/install)
* [Python](https://www.python.org/)
* [toga framework prerequisites (note: not the framework itself, that will be discussed later)](https://github.com/beeware/toga)
* [Poetry (if you're building on Linux)](https://python-poetry.org/docs/)

### Setting up the development environment

These instructions are written for Linux, their Windows equivalent is however very similar.

First, it's necessary to clone the repository and setup the virtual environment for any operations:

```
git clone https://github.com/RastislavKish/sdam
cd sdam/gui_py # This is the root folder for packages related to the Python graphical interface
python3 -m venv venv
. venv/bin/activate # venv\\scripts\activate on Windows
```

If everything is successful, the command line input should be prepended by (venv) mark. Now, it's time to install some development tools:

```
pip3 install maturin briefcase
pip3 install patchelf # you can omit this on Windows
```

Maturin is a tool for converting a PyO3 Rust library into an installable Python package, briefcase is a program we will use for compiling the whole project into an installable application.

Now, we need to compile several local dependencies. Most importantly the gui_py package, which provides Python access to the underlying Rust core code, but also a development version of Toga (the stable-one at this point suffers from several bugs that have been fixed in the meantime), also SDAM needs its own package for communicating with Speech Dispatcher if you're on Linux.

All of these deps are included in the repository, so they just need to be built into .whl files. Create a wheels directory:

```
mkdir wheels # This dir is included in the .gitignore, so whatever you put in is not going to be included in the repo if you make any commits
```

Here you will copy all the *.whl output from the sections below.

#### gui_py

```
cd gui_py
maturin build --release
cp target/wheels/*.whl ../wheels
cd ..
```

### Toga dev

```
mkdir toga # This is also included in .gitignore
cd toga
git clone https://github.com/beeware/toga
pip3 wheel ./toga/core ./toga/gtk # Replace gtk by winforms if you're on Windows
cp *.whl ../wheels
cd ..
```

#### speechd

```
cd speechd
poetry env use /usr/bin/python3
poetry build
cp dist/*.whl ../wheels
cd ..
```

### Updating the briefcase configuration

Now that dependencies are compiled, they need to be referred to by the briefcase configuration.

```
cd sdam
```

Edit the pyproject.toml file and wherever you spot ../wheels/, update the .whl file name with the one corresponding with the files you got from the build processes. The names will likely differ just in the Python version used. Note you don't need to update files that are not used by your platform.

### Build, run and package

Everything is in place, now all that remains is to perform a build, test everything by running the result and, eventually, packaging the program.

```
briefcase build -r
briefcase run
briefcase package
```

## A word on the project development architecture

In case anyone wanted to play with the code, here is a really brief and high-level overview of the project architecture.

Generally, SDAM consists of a:

* Graphical frontend, written in Python
* Rust core

The graphical frontend is a Briefcase app using the Toga GUI framework, which manages interaction with the user (not anything else). It includes a PyO3 package called gui_py, which is supposed to represent a SDAM document, which is functionally self-contained i.e. it can start recording into itself, stop recording into itself, start, pause and otherwise control playback, provide information about marks etc. Although, not all concepts of SDAM are reflected in its core, for example, timetravel is technically implemented as playback happening at the same time as recording, however the core does not implement it as a separate functionality, it's upto the GUI to control both activities and present them as a consistent function to the user.

At this moment, gui_py can work with only one document at a time, just like the whole GUI, and the interface are simple top-level functions.

The SDAM core library consists of two important parts:

* a Sdam structure, intended to represent a SDAM document for Rust frontends
* AudioHandler actor, this is basically the actual implementation of all the functions the Sdam structure provides, internally, Sdam always starts this actor when it's created and then just communicates with it through messages. The actor is called AudioHandler for historical reasons, it will likely eventually get renamed when I come up with a less confusing name. Either way, this nor any other actor are not intended to be used by anything outside the core library, i.e. they're completely internal implementation detail.

## License

Copyright (C) 2024 Rastislav Kish

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, version 3.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.

