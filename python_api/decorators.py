"""Python API decorators for easy custom node creation.

Users can create new nodes by simply decorating a class:

    from retro_music_maker import node, input, output

    @node(name="My Synth", category="Synthesizer")
    class MySynth(Node):
        freq = input(float, default=440.0)
        audio = output(float)

        def process(self, block_size):
            # Self.freq gives the control value
            # self.audio.set_result(samples)
            pass
"""
import numpy as np
from core.node import Node
from core.port import AudioPort, ControlPort, TriggerPort
from core import NodeRegistry


def node(name=None, category="General", description=""):
    """Decorator to register a class as a node."""
    def decorator(cls):
        cls._is_node = True
        cls._node_name = name or cls.__name__
        cls._node_category = category
        cls._node_description = description
        return cls
    return decorator


def input_type(type_name, default=None):
    """Define an input port."""
    def decorator(attr_name):
        return attr_name
    return decorator


def output_type(type_name):
    """Define an output port."""
    def decorator(attr_name):
        return attr_name
    return decorator


def create_node_class(base_class, name=None, category="General",
                      description=""):
    """Programmatically create a registered node class."""
    node_class = type(
        name or base_class.__name__,
        (base_class,),
        {
            "_is_node": True,
            "_node_name": name or base_class.__name__,
            "_node_category": category,
            "_node_description": description,
        }
    )
    NodeRegistry.register(node_class)
    return node_class


class SimpleNode(Node):
    """
    Base class for simple nodes with automatic port creation.

    Define ports as class attributes:
    - freq = FloatInput(default=440.0)
    - audio = AudioOutput()
    """

    def __init__(self, node_id=None):
        super().__init__(node_id)

    def _setup_ports(self):
        """Override to define ports using class attributes."""
        for attr_name in dir(self.__class__):
            attr = getattr(self.__class__, attr_name)
            if isinstance(attr, FloatInput):
                self._params[attr_name] = attr.default
            elif isinstance(attr, AudioInput):
                self._inputs[attr_name] = AudioPort(attr_name, self.id)
            elif isinstance(attr, AudioOutput):
                self._outputs[attr_name] = AudioPort(attr_name, self.id)
            elif isinstance(attr, ControlInput):
                self._inputs[attr_name] = ControlPort(attr_name, self.id)


class FloatInput:
    """Descriptor for float input parameters."""
    def __init__(self, default=0.0):
        self.default = default

    def __set_name__(self, owner, name):
        pass


class AudioInput:
    """Descriptor for audio input ports."""
    pass


class AudioOutput:
    """Descriptor for audio output ports."""
    pass


class ControlInput:
    """Descriptor for control input ports."""
    pass
