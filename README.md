# Game Boy Color Emulator

This is a emulator written for the Game Boy Color in Rust. As there are many emulators for the Game Boy Color out there, this was a passion project just for the enjoyment of creating one. Therefore, the bulk of this README will focus on the architecture choices and implementation details for other developers who are making emulators of their own and want to compare and contrast in addition to a history section detailing previous choices. There will also be a small section at the bottom detailing operation details, should you want to actually run it.

## Preface

I made this emulator with a small extra challenge in that I would only utilize resources describing the hardware of the Game Boy - no existing emulators, no threads on Reddit discussing emulator techniques, etc. This, combined with the fact I had minimal hardware and Rust knowledge when I started this project resulted in multiple rewrites of the overall structure and very likely resulted in some idiosyncratic (and possibly bad!) code. I can only apologize.

## Emulator Design Overview

The various components (CPU, APU, etc) are separated into seven structs contained inside one struct for the emulator as a whole. Due to the fact that Rust only allows one mutable reference at a time, nearly all the functions associated with each component are actually implemented on the Emulator struct since it has ownership of all of the component structs. However, I will refer to functions belonging to the component (even though they're implemented for the Emulator struct) as it makes more intuitive sense.

### Throttling

In order to properly throttle the emulator from running too fast, I utilized something I heard called 'sawtooth emulation' (full disclosure, this is one of the few times where the no non-hardware resource rule was broken, I saw this idea on a Reddit thread when I googling about clearing up a particular hardware behavior. See the history section for what happened before this). The emulator does 1/64th of a second's worth of work (15.625ms) and then spins until that period has passed. I refer to these work/wait sections as 'periods'.

### Advancing

Each component has an advance function that moves it forward one M-cycle. The emulator has a loop that calls the advance function for each component sequentially until it has done the appropriate amount of cycles for a period.

### Accuracy

The emulator is certainly not cycle-perfect, but it's not horrifically inaccurate as well. It does pass a number of accuracy tests, although fails a great number of the nitty-gritty ones. It does capture some of the obscure behaviors (HALT bug, various APU oddities). The specific component sections will go into more detail.

### Crates

The audio backend, the window handling, and the event/keyboard capture all utilize [SDL](https://docs.rs/sdl2/latest/sdl2/) as the backend. For drawing to the screen, [Pixels](https://docs.rs/pixels/latest/pixels/) is used for the drawing on the window. [RFD](https://docs.rs/rfd/latest/rfd/) is used for generating file dialog windows for opening roms/saves and saving states. [Serde](https://serde.rs/) and [Bincode](https://docs.rs/bincode/latest/bincode/) are used for serializing the save data.

## CPU

The CPU is relatively straightforward. It has an array of function pointers it uses to dispatch the command as opposed to one massive match statement. In terms of cycle accuracy, it does wait the appropriate amount of cycles for each action (and will adjust accordingly for things like taking/not taking a jump), but it is not accurate within instructions. Meaning it performs the operation all at once and then waits - it will not perform sub-actions like memory accesses, etc. accurately within that cycle count.

## PPU

Similar to the CPU, the PPU is cycle accurate at the mode level (VBLANK, etc), but not at the sub-actions within each mode. Although, unlike the CPU, it does not perform the actions all in one pass. In order to prevent some awkward bottlenecks, it spreads out the actions of each mode across the cycles. This equates to 10 OAM sprites searched per M-cycle in mode 2 and 1 background/window tile line or sprite line drawn per M-cycle in mode 3. The rendering occurs during the first M-Cycle in VBLANK.

## EPU

EPU stands for Event Processing Unit and it's entirely a fictional creation for the purposes of the emulator. This listens to key presses and window handling (resizing, exiting, etc.). However, the name is somewhat of a misnomer as it does not handle key-press events. Due to issues with the delay between subsequent events being sent out for a single key-press, it instead grabs the current keyboard state and then notes the keys pressed, rather than doing an event-based approach.

## Timer

There's honestly not much interesting to say about the timer. It's short and functions exactly how you'd expect.

## Memory

I've seen some people call this the MMU (Memory Management Unit), but I ended up referring to it as the Memory Access Unit (MAU). It takes in read/write requests from other components and returns data appropriate (just via a big match statement). Each request comes with a source parameter, an enum denoting which part of the program asked (so PPU can always access VRAM, but CPU may not at times, etc).

## APU

The APU was undoubtedly the bane of my existence. Due to it having probably worst documentation of all the components and my own complete lack of audio processing knowledge, it had more rewrites than any part of the system. The current method uses a queue based system. At the beginning of each period, the buffer size of the queues are checked - if any are zero, the queue is turned off so it can accumulate data. If it is currently off and the queues are non-zero, then sound is resumed. This turning-off-to-build-data only really needs to happen for the first period on boot, but it's checked at the beginning of each period just in case.

## History

### Multi-threading

Initially, the first designs of the emulator were all multi-threaded. Each component operated independently on its own thread. **Very** early on, the idea for memory sharing across these threads was using messaging passing approach (using Rust's MPSC) with the memory component acting like a server. Components would make memory access requests on a single channel and the memory component would reply on individual channels for each component. This was scrapped very early on as being not nearly performant enough. It soon moved to using mutexes for each memory component (ROM, VRAM, etc) (including a mutex for every IO register in order to reduce lock waiting times!). In order to properly time, the components would sleep (later changed to spinning as the sleep time on Windows was just too inaccurate) the appropriate time after each operation they performed.

The principal issue with this approach was that the components require a lot more close coordination than operating as independent threads could offer. What made it incredibly tricky to debug was that this approach *somewhat* worked. It was able to get through the boot ROM and only ran into issues later on, and it wasn't very apparent what the issue was. There were further attempts to fix this - one was tying everything to the CPU. Each thread would sleep after performing an operation and would have a condition variable that would wake it up whenever the CPU increased the cycle count (from performing an operation). If the cycle count met or exceeded the number of cycles assigned to that component's action, it would continue, otherwise it would go back to sleep. This, as you can imagine, did not solve matters. Not only was there coordination issues, but the CPU was sleeping after each *instruction*, which is way too granular. Eventually, it was all changed to the current method of single threaded and calling an advance function for each component.

### Graphics

Initially, everything graphics-wise was handled via SDL. Since the Game Boy handles drawing at the pixel level, this meant a lot of calls to draw a single point (or, being much more performative, an iterator of points). A big performance bump came from switching over the graphics handling to a very neat Rust library called Pixels (see the Crates section above). This was also much more intuitive for drawing on a pixel level (as you can imagine from the library name).

### Audio

Audio was always a big question mark. In graphics, you have a lot more leeway in the sense that you can stop providing graphics updates for a few milliseconds and the user won't be able to tell. However, gaps in audio will create popping sounds as it fills them with silence. Due to the method of using sawtooth emulation, this poses an issue as we're going to be sleeping/spinning for the great majority of each period. The first iteration of this used SDL callbacks. The callbacks would advance the square waves/wave ram pointer/white noise generator/etc. accordingly for each callback so things would advance as expected even during the sleeping time since the callbacks are their own separated threads (memory was shared between the callbacks and the main thread with mutexes and some atomic types).

This has a drawback, however, as since the majority of time is spent sleeping, the callbacks will essentially operate on whatever was the last settings set by the main thread. And because we compress about 15ms of work into only a few milliseconds and then sleep, we could miss out on sound details. For example, if the program wanted one frequency (Freq A) to run for 7ms and then another (Freq B) for 7ms, the main thread change to Freq A at the beginning and then in very short order, change to Freq B due to how the time compressing works. Meaning it would likely just be Freq B for 14ms.

In practice, this had little to no effect on sound quality. Since periods are so short (~15ms), there wasn't really any detail that would happen at that granular a level that you would notice missing. However, it bothered me anyway, so I switched things over to a queue based system. A queue based system makes far more sense for this type of emulator as we simply generate ~15ms of audio in a few ms and then that is played out over the period. The issue that arose was we generate **exactly** 15.625ms of audio which leaves no room for timing being even slightly off or else we'll have pops in the audio. The solution was to turn off playback for a period whenever the buffer was empty (which will produce a pop), but will fill the buffer up with 15.625ms of audio which is more than enough to handle minor timing changes. (I also did some short Googling to confirm that you can't notice a 15ms audio lag and turns out you can't!)

## Resources

There were a number of great resources I used to help create this emulator.

### Documentation

The ubiquitously mentioned [Pan Docs](https://gbdev.io/pandocs/) were immensely helpful for all components of the emulator. The RGBDS [CPU documentation](https://rgbds.gbdev.io/docs/v0.4.2/gbz80.7) helped me with some additional details about opcodes (mostly about the differences between all the various rotation and shift opcodes). The [Game Boy CPU Manual](http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf) was used to fill in some small gaps for questions about (DMG only) CPU behavior. I got most of my audio information from the Game Boy Development wiki [page on it](https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware).

### Software

[RGBDS](https://rgbds.gbdev.io/) itself was used occasionally to make some custom test ROMs to test certain behavior. [MGBDIS](https://github.com/mattcurrie/mgbdis) was used to disassemble some less popular games for debugging that didn't have a disassembly repo on GitHub.

### Test ROMs

All of the [Blargg test ROMs](https://github.com/retrio/gb-test-roms) were helpful, but especially the sound and CPU instruction ones. For graphics, the [cgb-acid2](https://github.com/mattcurrie/cgb-acid2) test ROM (and it's [corresponding DMG one](https://github.com/mattcurrie/dmg-acid2)) were instrumental in nailing all of the various graphics quirks that come with the Game Boy.
