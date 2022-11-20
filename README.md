
# Dradu

The goal of this project is to make a FOSS virtual tabletop tool which is able
to run both natively and in browser - make a better replacement for those
proprietary and laggy VTTs


## Current state

This project is SO raw it's not even usable, however, this
should change in 2-3 months or so. The main reason for uploading this to GH is
just to have a code backup and be able to work on this from different machines.
You really shouldn't look into this repo right now, but maybe after some time
this project will actually be worth something


## Running and using the program

### Setup
 1. CD into **server** directory and run `./main.py`
  (If it doesn't work - try using python3.9 or newer)
 2. Open another terminal, CD into **client** directory and run the client
  using `cargo run --release` (It may take a while to compile)
 3. In the client, enter servers IP address and press *"New game"*. Server
  should've outputted its IP to the terminal when you launched it
 4. To connect another client to the room you've just created, click on the *"Info"*
  tab and copy room address using the *"Copy"* button. Paste it into another
  client and hit *"Join"*

### Adding assets

Only GM can add assets to the scene. They are stored locally on your
computer in `~/.local/share/dradu/assets`. You can move them to this directory
or open *"Assets"* tab and simply drag-and-drop them into it - Dradu will
copy dropped assets into its local storage. To add asset to the scene, simply
click on its name in *"Assets"* tab. You can right-click it and set it
as background image

### Chat commands

They are implemented on the server side. Current server supports the following
commands:

 - `/nickname <NAME>` `/nick <NAME>` - Change your nickname
 - `/color <R> <G> <B>` - Change your nickname color. RGB values are from 0 to 255
