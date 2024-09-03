rysyn
======

Rysyn is a simple Python library for creating and manipulating 2D and 3D
shapes. It is built on top of the `pygame` library, and is designed to be
easy to use and understand.

Rysyn is currently in development and is not yet ready for use in
production. However, it is already being used in a number of personal
projects, and we welcome contributions from the community.

Installation
------------

Rysyn can be installed using pip:

```
pip install rysyn
```

Usage
---------

To use Rysyn, you first need to create a `Window` object:

```python
from rysyn import Window

window = Window()
```

You can then create shapes and add them to the window:

```python
from rysyn import Circle, Rectangle

circle = Circle((100, 100), 50)
rectangle = Rectangle((200, 200), (100, 50))

window.add(circle)
window.add(rectangle)
```

You can also set the background color of the window:

```python
window.background_color = (255, 255, 255)
```

Finally, you can start the window's main loop:

```python
window.run()
```