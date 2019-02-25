
# izuna / いずな

A <del>fake</del> mouse driver for everyone without a mouse but a keypad.

![](images/izuna.jpg)

This program allows you to control the system cursor via a keyboard. Most
functions on the mouse are replicated by this driver in an apparent way.
Reasonably, this certain program should be more friendly to TouchStyk users,
but can be easy to use to other groups as well.

## Installation

```sh
pip install -r requirements.txt
python izuna.py  # Use python3
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

-   [ ] one can only stop the program through a task manager
-   [x] <del>*izuna* isn't as cute as *shiro* is</del>
