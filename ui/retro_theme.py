"""Retro NES-style color theme for the GUI."""
import dearpygui.dearpygui as dpg


class RetroTheme:
    """NES-inspired color palette for the Retro Music Maker GUI."""

    # Background colors
    BG_DARK = [0.102, 0.102, 0.180, 1.0]       # #1a1a2e
    BG_MID = [0.086, 0.149, 0.224, 1.0]         # #16213e
    BG_PANEL = [0.125, 0.125, 0.200, 1.0]        # #202033
    BG_INPUT = [0.15, 0.15, 0.25, 1.0]           # inputs

    # Text colors
    TEXT_WHITE = [0.918, 0.918, 0.918, 1.0]      # #eaeaea
    TEXT_DIM = [0.6, 0.6, 0.65, 1.0]             # dim text
    TEXT_RED = [0.918, 0.271, 0.251, 1.0]        # errors
    TEXT_GREEN = [0.063, 0.780, 0.604, 1.0]      # success
    TEXT_YELLOW = [0.961, 0.651, 0.137, 1.0]     # warnings

    # Accent
    ACCENT_RED = [1.0, 0.0, 0.251, 1.0]          # #ff0040
    ACCENT_HOVER = [1.0, 0.1, 0.35, 1.0]         # brighter red on hover
    ACCENT_ACTIVE = [1.0, 0.2, 0.45, 1.0]         # active state

    # Node category colors (border glow)
    CAT_OSCILLATOR = [0.914, 0.271, 0.376, 1.0]   # red/pink
    CAT_FILTER = [0.059, 0.204, 0.376, 1.0]        # blue
    CAT_ENVELOPE = [0.063, 0.780, 0.604, 1.0]     # green
    CAT_EFFECTS = [0.961, 0.651, 0.137, 1.0]      # orange
    CAT_MIXER = [0.314, 0.510, 0.910, 1.0]        # light blue
    CAT_INPUT = [0.553, 0.271, 0.855, 1.0]         # purple
    CAT_OUTPUT = [0.918, 0.918, 0.918, 1.0]        # white
    CAT_SYNTH = [0.933, 0.545, 0.341, 1.0]         # brown/orange
    CAT_SEQUENCER = [0.0, 0.600, 0.800, 1.0]       # teal

    # Port colors
    PORT_AUDIO = [0.918, 0.918, 0.918, 1.0]        # white
    PORT_CONTROL = [0.961, 0.800, 0.137, 1.0]      # yellow
    PORT_TRIGGER = [0.063, 0.780, 0.604, 1.0]      # green

    # Connection line
    CONNECTION = [0.914, 0.271, 0.376, 1.0]         # red/pink
    CONNECTION_HIGHLIGHT = [1.0, 0.4, 0.6, 1.0]      # bright pink

    # Scrollbar
    SCROLLBAR_BG = [0.15, 0.15, 0.25, 0.5]
    SCROLLBAR_THUMB = [0.4, 0.4, 0.5, 0.8]

    # Canvas grid
    GRID_COLOR = [0.2, 0.2, 0.3, 0.3]

    NODE_CATEGORIES = {
        "Oscillator": CAT_OSCILLATOR,
        "Synthesizer": CAT_SYNTH,
        "Filter": CAT_FILTER,
        "Envelope": CAT_ENVELOPE,
        "Effects": CAT_EFFECTS,
        "Mixer": CAT_MIXER,
        "Input": CAT_INPUT,
        "Output": CAT_OUTPUT,
        "Sequencer": CAT_SEQUENCER,
    }

    @classmethod
    def apply(cls):
        """Apply the retro theme to the Dear PyGui app."""
        # DPY 2.0: styling done per-item via dpg.configure_item
        return {}
