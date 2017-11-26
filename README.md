
# izuna / いずな

A <del>fake</del> mouse driver for everyone without a mouse.

![](images/izuna.jpg)

This program allows you to control the system cursor via a keyboard. Most
functions on the mouse are replicated by this driver in an apparent way.
Reasonably, this certain program should be more friendly to TouchStyk users,
but can be easy to use to other groups as well.

Currently it only works under Python 2.7.

## Installation

```sh
pythonw izuna.py # Use python2
```

## Usage

The keys and their corresponding functions are as following:

| key          | function                                 |
| ------------ | ---------------------------------------- |
| Numpad 1     | move / lower-left                        |
| Numpad 2     | move / lower                             |
| Numpad 3     | move / lower-right                       |
| Numpad 4     | move / left                              |
| Numpad 6     | move / right                             |
| Numpad 7     | move / upper-left                        |
| Numpad 8     | move / upper                             |
| Numpad 9     | move / upper-right                       |
| Numpad 0     | left click (holdable)                    |
| Numpad Enter | left click (holdable)                    |
| Numpad 5     | brake while cursor is moving, left click (holdable) while static |
| Numpad +     | right click                              |
| Numpad /     | middle click                             |
| Numpad *     | scroll up                                |
| Numpad -     | scroll down                              |
| Num Lock     | *izuna* is disabled while num lock is on, vice versa |

## Known Issues

-   [ ] *izuna* might not work properly on multi displays
-   [ ] the speed configurations might require optimizations or customizations
-   [ ] pyHook in Python3 has some serious issues which causes the program to terminate unexpectedly, so we are using Python2 instead, until the bug is resolved
-   [ ] one can only stop the program through a task manager
-   [x] <del>*izuna* isn't as cute as *shiro* is</del>