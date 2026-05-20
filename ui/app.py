"""Tkinter-based node graph GUI for Retro Music Maker.

Built on the ComfyUI approach - using tkinter.Canvas for the node
graph surface with custom rendering of nodes, ports, and connections.
"""
import sys
import os
import json
import importlib.util
import tkinter as tk
from tkinter import ttk, filedialog, messagebox
import numpy as np
import threading
import queue

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from core.engine import Engine
from core import NodeRegistry
from ui.midi_input import MIDIInput


# =============================================================================
# Color Theme - Professional dark palette (Blender/Unreal inspired)
# =============================================================================
class Theme:
    BG = "#1a1b26"
    BG_DARK = "#13141e"
    BG_PANEL = "#1e2030"
    BG_PANEL_ALT = "#24263a"
    BG_INPUT = "#292b40"
    BG_HOVER = "#333654"
    BORDER = "#3d4060"
    BORDER_LIGHT = "#4d5070"
    TEXT = "#c0caf5"
    TEXT_DIM = "#565a7e"
    TEXT_BRIGHT = "#e1e4ff"
    ACCENT = "#7aa2f7"
    ACCENT_DARK = "#1e3a6e"
    GREEN = "#9ece6a"
    RED = "#f7768e"
    YELLOW = "#e0af68"
    TEAL = "#73daca"
    MAGENTA = "#bb9af7"
    ORANGE = "#ff9e64"

    CAT_COLORS = {
        "Oscillator": "#7aa2f7",
        "Synthesizer": "#e0af68",
        "Filter": "#73daca",
        "Envelope": "#9ece6a",
        "Effects": "#e0af68",
        "Mixer": "#bb9af7",
        "Input": "#bb9af7",
        "Output": "#c0caf5",
        "Sequencer": "#7dcfff",
    }

    PORT_COLORS = {
        "AudioPort": "#c0caf5",
        "ControlPort": "#e0af68",
        "TriggerPort": "#9ece6a",
    }

    NODE_W = 200
    NODE_H_MIN = 80
    PORT_R = 5
    PORT_GAP = 26
    TITLE_H = 26
    HEADER_BAR = 4


# =============================================================================
# Graph Canvas
# =============================================================================
class GraphCanvas(tk.Canvas):
    def __init__(self, parent, **kwargs):
        super().__init__(parent, bg=Theme.BG, highlightthickness=0, **kwargs)
        self.nodes = {}
        self.connections = []
        self.selected_node = None
        self.dragging_node = None
        self.connecting = None
        self.connect_line = None
        self.panning = False
        self.pan_start = None
        self.zoom = 1.0
        self._scroll_region = (0, 0, 5000, 4000)
        self.configure(scrollregion=self._scroll_region)

        # Scrollbars on parent, not self
        self.parent_frame = parent

        self.bind("<Button-1>", self._on_left_click)
        self.bind("<B1-Motion>", self._on_drag)
        self.bind("<ButtonRelease-1>", self._on_release)
        self.bind("<Button-3>", self._on_right_click)
        self.bind("<B3-Motion>", self._on_pan)
        self.bind("<MouseWheel>", self._on_zoom)
        self.bind("<Double-Button-1>", self._on_double_click)
        self.bind("<Delete>", self._on_delete)
        self._parent = parent

    def add_node(self, node_obj, x, y):
        graph_node = GraphNode(self, node_obj, x, y)
        self.nodes[node_obj.id] = graph_node
        return graph_node

    def remove_node(self, node_id):
        if node_id in self.nodes:
            node = self.nodes[node_id]
            node.destroy()
            del self.nodes[node_id]
            self.connections = [c for c in self.connections
                                if c.src_id != node_id and c.dst_id != node_id]
            self._redraw_connections()

    def connect_ports(self, src_id, src_port, dst_id, dst_port):
        self.connections = [c for c in self.connections
                            if not (c.src_id == src_id and c.dst_id == dst_id
                                    and c.src_port == src_port)]
        conn = GraphConnection(self, src_id, src_port, dst_id, dst_port)
        self.connections.append(conn)
        conn.draw()

    def disconnect(self, src_id, dst_id):
        self.connections = [c for c in self.connections
                            if not (c.src_id == src_id and c.dst_id == dst_id)]
        self._redraw_connections()

    def get_port_screen_pos(self, node_id, port_name, is_output):
        if node_id not in self.nodes:
            return None
        node = self.nodes[node_id]
        x, y = node.get_port_position(port_name, is_output)
        return self.canvasx(x), self.canvasy(y)

    def _on_left_click(self, event):
        if self.connecting:
            hit = self._hit_test_port(event.x_root, event.y_root)
            if hit and hit != self.connecting:
                src_id, src_port, _ = self.connecting
                dst_id, dst_port, _ = hit
                self.connecting = None
                self._cancel_connect_line()
                try:
                    self._parent.on_connect_ports(src_id, src_port, dst_id, dst_port)
                except Exception:
                    pass
            else:
                self.connecting = None
                self._cancel_connect_line()
            return
        if self.panning:
            return
        hit_node = self._hit_test_node(event.x, event.y)
        if hit_node:
            hit_port = self._hit_test_node_port(hit_node, event.x, event.y)
            if hit_port:
                self.connecting = (hit_node.node_id, hit_port["port_name"], hit_port["type"])
                self._create_connect_line(hit_node, hit_port)
            else:
                self.select_node(hit_node)
                self.dragging_node = hit_node
                self.drag_offset = (event.x - hit_node.x, event.y - hit_node.y)
        else:
            self.deselect_all()

    def _on_drag(self, event):
        if self.dragging_node:
            dx = event.x - self.dragging_node.x
            dy = event.y - self.dragging_node.y
            self.dragging_node.move(dx - self.dragging_node._last_dx,
                                    dy - self.dragging_node._last_dy)
            self.dragging_node._last_dx = dx
            self.dragging_node._last_dy = dy
            self._redraw_connections()
        elif self.connecting:
            self._update_connect_line(event.x, event.y)

    def _on_release(self, event):
        self.dragging_node = None
        if self.panning:
            self.panning = False

    def _on_right_click(self, event):
        self.panning = True
        self.pan_start = (event.x, event.y)

    def _on_pan(self, event):
        if not self.panning:
            return
        self.scan_dragto(event.x, event.y, gain=1)

    def _on_zoom(self, event):
        delta = -0.1 if event.delta > 0 else 0.1
        self.zoom = max(0.5, min(2.0, self.zoom + delta))

    def _on_double_click(self, event):
        hit_node = self._hit_test_node(event.x, event.y)
        if hit_node:
            self._parent.on_node_double_click(hit_node.node_obj)

    def _on_delete(self, event):
        if self.selected_node:
            self._parent.on_delete_node(self.selected_node.node_id)

    def _hit_test_node(self, x, y):
        for nid, node in self.nodes.items():
            if node.contains(x, y):
                if not self._hit_test_node_port(node, x, y):
                    return node
        return None

    def _hit_test_node_port(self, node, x, y):
        for is_output, port_name in node._get_all_ports():
            px, py = node.get_port_position(port_name, is_output)
            cx, cy = self.canvasx(px), self.canvasy(py)
            dist = ((x - cx) ** 2 + (y - cy) ** 2) ** 0.5
            if dist <= Theme.PORT_R + 6:
                port_type = node._get_port_type(port_name, is_output)
                return {"port_name": port_name, "type": port_type, "is_output": is_output}
        return None

    def _hit_test_port(self, root_x, root_y):
        for nid, node in self.nodes.items():
            for is_output, port_name in node._get_all_ports():
                px, py = node.get_port_position(port_name, is_output)
                cx, cy = self.canvasx(px), self.canvasy(py)
                if ((root_x - cx) ** 2 + (root_y - cy) ** 2) ** 0.5 <= Theme.PORT_R + 8:
                    port_type = node._get_port_type(port_name, is_output)
                    return (nid, port_name, port_type)
        return None

    def select_node(self, node):
        self.deselect_all()
        self.selected_node = node
        node.select()

    def deselect_all(self):
        if self.selected_node:
            self.selected_node.deselect()
        self.selected_node = None

    def _create_connect_line(self, node, port_info):
        sx, sy = node.get_port_position(port_info["port_name"], port_info["is_output"])
        self.connect_line = self.create_line(sx, sy, sx, sy,
                                             fill=Theme.ACCENT, width=2, dash=(6, 3))
        self.tag_raise(self.connect_line)

    def _update_connect_line(self, mx, my):
        if self.connect_line and self.connecting:
            nid = self.connecting[0]
            port_name = self.connecting[1]
            is_output = next(
                (o for o, n in self.nodes[nid]._get_all_ports() if n == port_name), False
            )
            sx, sy = self.nodes[nid].get_port_position(port_name, is_output)
            self.coords(self.connect_line, sx, sy, mx, my)

    def _cancel_connect_line(self):
        if self.connect_line:
            self.delete(self.connect_line)
            self.connect_line = None

    def _redraw_connections(self):
        for conn in self.connections:
            conn.redraw()

    def redraw_all(self):
        for node in self.nodes.values():
            node.redraw()
        self._redraw_connections()


# =============================================================================
# Graph Node
# =============================================================================
class GraphNode:
    def __init__(self, canvas, node_obj, x, y):
        self.canvas = canvas
        self.node_obj = node_obj
        self.x = x
        self.y = y
        self.selected = False
        self._last_dx = x
        self._last_dy = y
        self._items = {}
        # Measure height needed for ports
        self._port_count = len(self._get_all_ports())
        self._calc_height()
        self._draw()

    def _calc_height(self):
        h = Theme.TITLE_H + Theme.HEADER_BAR
        for is_output, name in self._get_all_ports():
            h += Theme.PORT_GAP
        h += 12
        self.h = max(h, Theme.NODE_H_MIN)

    def _draw(self):
        w = Theme.NODE_W
        cat = getattr(self.node_obj, 'category', 'General')
        color = Theme.CAT_COLORS.get(cat, Theme.TEXT_DIM)
        dark = self._hex_to_rgb(color)
        dark = (dark[0] * 0.3, dark[1] * 0.3, dark[2] * 0.3)
        dark_hex = f"#{int(dark[0]*255):02x}{int(dark[1]*255):02x}{int(dark[2]*255):02x}"

        # Shadow
        self._items["shadow"] = self.canvas.create_rectangle(
            self.x + 3, self.y + 3, self.x + w + 3, self.y + self.h + 3,
            fill="#00000050", outline="", tags=(self.node_obj.id, "node"))

        # Body
        self._items["body"] = self.canvas.create_rectangle(
            self.x, self.y, self.x + w, self.y + self.h,
            fill=Theme.BG_PANEL, outline=BORDER, width=1,
            tags=(self.node_obj.id, "node"))

        # Header bar (thin color accent)
        self._items["header"] = self.canvas.create_rectangle(
            self.x, self.y, self.x + w, self.y + Theme.HEADER_BAR,
            fill=color, outline="", tags=(self.node_obj.id, "header"))

        # Title bar bg
        title_y = self.y + Theme.HEADER_BAR
        self._items["title_bg"] = self.canvas.create_rectangle(
            self.x, title_y, self.x + w, title_y + Theme.TITLE_H,
            fill=dark_hex, outline="", tags=(self.node_obj.id, "title"))

        # Title text
        self._items["title_text"] = self.canvas.create_text(
            self.x + 8, title_y + Theme.TITLE_H // 2,
            text=node_obj.id.replace("_", " ").title()[:20],
            fill=Theme.TEXT_BRIGHT, font=("Segoe UI", 10, "bold"), anchor="w",
            tags=(self.node_obj.id, "title_text"))

        # Draw ports
        self._draw_ports()

    def _draw_ports(self):
        left_ports = []
        right_ports = []
        for is_output, port_name in self._get_all_ports():
            if is_output:
                right_ports.append((port_name, is_output))
            else:
                left_ports.append((port_name, is_output))

        # Draw left ports
        start_y = self.y + Theme.HEADER_BAR + Theme.TITLE_H + 2
        for i, (name, is_out) in enumerate(left_ports):
            py = start_y + i * Theme.PORT_GAP
            self._draw_port(is_out, name, self.x, py)

        # Draw right ports
        for i, (name, is_out) in enumerate(right_ports):
            py = start_y + i * Theme.PORT_GAP
            self._draw_port(is_out, name, self.x + Theme.NODE_W, py)

    def _draw_port(self, is_output, port_name, x, y):
        port_type = self._get_port_type(port_name, is_output)
        color = Theme.PORT_COLORS.get(port_type, Theme.TEXT_DIM)

        # Port circle
        self._items[f"port_{port_name}"] = self.canvas.create_oval(
            x - Theme.PORT_R, y - Theme.PORT_R,
            x + Theme.PORT_R, y + Theme.PORT_R,
            fill=color, outline="", tags=("port", self.node_obj.id, port_name))

        # Port label
        if is_output:
            label_x = x + Theme.PORT_R + 6
            anchor = "w"
        else:
            label_x = x - Theme.PORT_R - 6
            anchor = "e"

        self.canvas.create_text(
            label_x, y,
            text=port_name,
            fill=Theme.TEXT_DIM, font=("Segoe UI", 9),
            anchor=anchor)

    def _get_all_ports(self):
        result = []
        if hasattr(self.node_obj, 'outputs'):
            for name in self.node_obj.outputs:
                result.append((True, name))
        if hasattr(self.node_obj, 'inputs'):
            for name in self.node_obj.inputs:
                result.append((False, name))
        return result

    def _get_port_type(self, port_name, is_output):
        try:
            port = (self.node_obj.outputs[port_name] if is_output
                    else self.node_obj.inputs[port_name])
            return port.__class__.__name__
        except (KeyError, AttributeError):
            return "AudioPort"

    def get_port_position(self, port_name, is_output):
        idx = 0
        for i, (o, n) in enumerate(self._get_all_ports()):
            if n == port_name and o == is_output:
                idx = i
                break
        y = self.y + Theme.HEADER_BAR + Theme.TITLE_H + 2 + idx * Theme.PORT_GAP
        x = 0 if not is_output else Theme.NODE_W
        return self.x + x, y

    def contains(self, x, y):
        return (self.x - 3 <= x <= self.x + Theme.NODE_W + 3 and
                self.y - 3 <= y <= self.y + self.h + 3)

    def select(self):
        self.selected = True
        self.canvas.itemconfigure(self._items.get("body"), outline=Theme.ACCENT, width=2)
        self.canvas.itemconfigure(self._items.get("shadow"), fill="#7aa2f740")

    def deselect(self):
        self.selected = False
        cat = getattr(self.node_obj, 'category', 'General')
        color = Theme.CAT_COLORS.get(cat, Theme.TEXT_DIM)
        self.canvas.itemconfigure(self._items.get("body"), outline=color, width=1)
        self.canvas.itemconfigure(self._items.get("shadow"), fill="#00000050")

    def move(self, dx, dy):
        self.x += dx
        self.y += dy
        for item_id in self.canvas.find_withtag(self.node_obj.id):
            self.canvas.move(item_id, dx, dy)

    def destroy(self):
        for item_id in self.canvas.find_withtag(self.node_obj.id):
            self.canvas.delete(item_id)
        self._items.clear()

    def redraw(self):
        cat = getattr(self.node_obj, 'category', 'General')
        color = Theme.CAT_COLORS.get(cat, Theme.TEXT_DIM)
        if self._items.get("body"):
            self.canvas.itemconfigure(self._items["body"], outline=color)

    @staticmethod
    def _hex_to_rgb(hex_color):
        hex_color = hex_color.lstrip('#')
        return tuple(int(hex_color[i:i+2], 16) / 255.0 for i in (0, 2, 4))


# =============================================================================
# Graph Connection
# =============================================================================
class GraphConnection:
    def __init__(self, canvas, src_id, src_port, dst_id, dst_port):
        self.canvas = canvas
        self.src_id = src_id
        self.src_port = src_port
        self.dst_id = dst_id
        self.dst_port = dst_port
        self.line_item = None
        self.src_node_obj = None
        self.dst_node_obj = None

    def draw(self):
        pos1 = self.canvas.get_port_screen_pos(self.src_id, self.src_port, True)
        pos2 = self.canvas.get_port_screen_pos(self.dst_id, self.dst_port, False)
        if pos1 is None or pos2 is None:
            return

        cat = "default"
        for nid in [self.src_id, self.dst_id]:
            if nid in self.canvas.nodes:
                node = self.canvas.nodes[nid]
                if hasattr(node.node_obj, 'category'):
                    cat = node.node_obj.category
        color = Theme.CAT_COLORS.get(cat, Theme.ACCENT)

        # Smooth bezier curve
        dx = max(abs(pos2[0] - pos1[0]), 40)
        cp1x = pos1[0] + dx * 0.4
        cp2x = pos2[0] - dx * 0.4

        self.line_item = self.canvas.create_line(
            pos1[0], pos1[1], cp1x, pos1[1], cp2x, pos2[1], pos2[0], pos2[1],
            fill=color, width=2.5, smooth=True,
            tags=("connection", self.src_id, self.dst_id),
            capstyle="round")

    def redraw(self):
        if self.line_item is None:
            self.draw()
            return
        pos1 = self.canvas.get_port_screen_pos(self.src_id, self.src_port, True)
        pos2 = self.canvas.get_port_screen_pos(self.dst_id, self.dst_port, False)
        if pos1 is None or pos2 is None:
            return
        dx = max(abs(pos2[0] - pos1[0]), 40)
        cp1x = pos1[0] + dx * 0.4
        cp2x = pos2[0] - dx * 0.4
        self.canvas.coords(self.line_item,
                           pos1[0], pos1[1], cp1x, pos1[1],
                           cp2x, pos2[1], pos2[0], pos2[1])


# =============================================================================
# Main Application
# =============================================================================
class RetroApp:
    def __init__(self, root):
        self.root = root
        self.engine = Engine()
        self.node_counter = 0
        self._file_path = None
        self._console_queue = queue.Queue()
        self._midi_manager = None
        self._palette_visible = True
        self._console_visible = True

        self.root.title("Retro Music Maker")
        self.root.geometry("1440x860")
        self.root.minsize(960, 600)
        self.root.configure(bg=Theme.BG)

        self._setup_styles()
        self._create_menu()
        self._create_toolbar()
        self._create_main_layout()

        self._load_plugins()
        self._log("Retro Music Maker v1.0", "green")

        self.root.after(100, self._process_console_queue)

    def _setup_styles(self):
        style = ttk.Style()
        style.theme_use('clam')
        style.configure("TFrame", background=Theme.BG)
        style.configure("TLabel", background=Theme.BG, foreground=Theme.TEXT)
        style.configure("TButton", background=Theme.BG_PANEL, foreground=Theme.TEXT,
                        padding=(12, 6), relief="flat", font=("Segoe UI", 9))
        style.map("TButton", background=[("active", Theme.BG_HOVER)],
                  foreground=[("active", Theme.TEXT_BRIGHT)])
        style.configure("TLabelframe", background=Theme.BG,
                        bordercolor=Theme.BORDER)
        style.configure("TLabelframe.Label", background=Theme.BG,
                        foreground=Theme.TEXT, font=("Segoe UI", 9))
        style.configure("Horizontal.TScrollbar", background=Theme.BG_PANEL,
                        troughcolor=Theme.BG)

    def _create_menu(self):
        menubar = tk.Menu(self.root, bg=Theme.BG_DARK, fg=Theme.TEXT,
                         activebackground=Theme.BG_HOVER, activeforeground=Theme.TEXT_BRIGHT,
                         relief="flat")
        self.root.config(menu=menubar)

        file_menu = tk.Menu(menubar, tearoff=0, bg=Theme.BG_DARK, fg=Theme.TEXT,
                            activebackground=Theme.BG_HOVER, activeforeground=Theme.TEXT_BRIGHT)
        menubar.add_cascade(label="File", menu=file_menu)
        file_menu.add_command(label="New", accelerator="Ctrl+N", command=self._new_graph)
        file_menu.add_command(label="Open...", accelerator="Ctrl+O", command=self._open_dialog)
        file_menu.add_command(label="Save", accelerator="Ctrl+S", command=self._save_graph)
        file_menu.add_command(label="Save As...", command=self._save_as_dialog)
        file_menu.add_separator()
        file_menu.add_command(label="Export WAV", command=self._export_wav)
        file_menu.add_separator()
        file_menu.add_command(label="Quit", accelerator="Ctrl+Q", command=self.root.quit)

        edit_menu = tk.Menu(menubar, tearoff=0, bg=Theme.BG_DARK, fg=Theme.TEXT,
                            activebackground=Theme.BG_HOVER, activeforeground=Theme.TEXT_BRIGHT)
        menubar.add_cascade(label="Edit", menu=edit_menu)
        edit_menu.add_command(label="Delete Selected", accelerator="Del",
                              command=self._delete_selected)

        audio_menu = tk.Menu(menubar, tearoff=0, bg=Theme.BG_DARK, fg=Theme.TEXT,
                             activebackground=Theme.BG_HOVER, activeforeground=Theme.TEXT_BRIGHT)
        menubar.add_cascade(label="Audio", menu=audio_menu)
        audio_menu.add_command(label="Start", accelerator="Space", command=self._start_audio)
        audio_menu.add_command(label="Stop", accelerator="Esc", command=self._stop_audio)

        view_menu = tk.Menu(menubar, tearoff=0, bg=Theme.BG_DARK, fg=Theme.TEXT,
                            activebackground=Theme.BG_HOVER, activeforeground=Theme.TEXT_BRIGHT)
        menubar.add_cascade(label="View", menu=view_menu)
        view_menu.add_command(label="Palette", command=self._toggle_palette)
        view_menu.add_command(label="Console", command=self._toggle_console)

        self.root.bind("<Control-n>", lambda e: self._new_graph())
        self.root.bind("<Control-o>", lambda e: self._open_dialog())
        self.root.bind("<Control-s>", lambda e: self._save_graph())
        self.root.bind("<Delete>", lambda e: self._delete_selected())
        self.root.bind("<space>", lambda e: self._toggle_audio())
        self.root.bind("<Escape>", lambda e: self._stop_audio())

    def _create_toolbar(self):
        self.toolbar = ttk.Frame(self.root, padding=(6, 5))
        self.toolbar.grid(row=0, column=0, columnspan=2, sticky="ew")
        self.toolbar.configure(style="TFrame")

        # Title label
        ttk.Label(self.toolbar, text="Retro Music Maker",
                  font=("Segoe UI", 11, "bold"), foreground=Theme.TEXT_BRIGHT).pack(side=tk.LEFT, padx=(0, 15))

        ttk.Separator(self.toolbar, orient=tk.VERTICAL).pack(side=tk.LEFT, fill=tk.Y, padx=5)

        # File buttons
        self._btn_new = ttk.Button(self.toolbar, text="New", command=self._new_graph, width=10)
        self._btn_new.pack(side=tk.LEFT, padx=1)
        self._btn_open = ttk.Button(self.toolbar, text="Open", command=self._open_dialog, width=10)
        self._btn_open.pack(side=tk.LEFT, padx=1)
        self._btn_save = ttk.Button(self.toolbar, text="Save", command=self._save_graph, width=10)
        self._btn_save.pack(side=tk.LEFT, padx=1)
        self._btn_export = ttk.Button(self.toolbar, text="Export WAV", command=self._export_wav, width=12)
        self._btn_export.pack(side=tk.LEFT, padx=1)

        ttk.Separator(self.toolbar, orient=tk.VERTICAL).pack(side=tk.LEFT, fill=tk.Y, padx=8)

        self._audio_status = ttk.Label(self.toolbar, text="● Stopped", foreground=Theme.TEXT_DIM)
        self._audio_status.pack(side=tk.LEFT, padx=8)

        # Spacer
        ttk.Frame(self.toolbar, width=20).pack(side=tk.LEFT, fill=tk.X, expand=True)

        # View toggles
        self._btn_palette = ttk.Button(self.toolbar, text="Palette", command=self._toggle_palette, width=11)
        self._btn_palette.pack(side=tk.LEFT, padx=1)
        self._btn_console = ttk.Button(self.toolbar, text="Console", command=self._toggle_console, width=11)
        self._btn_console.pack(side=tk.LEFT, padx=1)

    def _create_main_layout(self):
        self.root.grid_rowconfigure(0, weight=0)
        self.root.grid_rowconfigure(1, weight=1)
        self.root.grid_rowconfigure(2, weight=0)
        self.root.grid_columnconfigure(0, weight=3)
        self.root.grid_columnconfigure(1, weight=1)

        # Left side (pack-based so palette expands)
        self.left_frame = ttk.Frame(self.root)
        self.left_frame.grid(row=1, column=0, sticky="nswe", padx=3, pady=3)
        self.left_frame.grid_rowconfigure(0, weight=1)
        self.left_frame.grid_columnconfigure(0, weight=1)

        # Palette (use pack so it takes available space)
        self.palette_container = ttk.Frame(self.left_frame)
        self.palette_container.pack(fill=tk.X, padx=0, pady=(0, 3))
        self.palette_frame = ttk.LabelFrame(self.palette_container, text="Nodes")
        self.palette_frame.pack(fill=tk.BOTH, expand=True, padx=4, pady=4)
        self._create_palette()

        # Canvas container
        self.canvas_wrapper = ttk.Frame(self.left_frame)
        self.canvas_wrapper.pack(fill=tk.BOTH, expand=True)
        self._create_canvas()

        # Right side
        self.right_frame = ttk.Frame(self.root)
        self.right_frame.grid(row=1, column=1, sticky="nswe", padx=3, pady=3)

        self.props_frame = ttk.LabelFrame(self.right_frame, text="Properties")
        self._create_properties()
        self.props_frame.pack(fill=tk.BOTH, expand=True)

        # Console
        self.console_frame = ttk.LabelFrame(self.root, text="Console")
        self._create_console()
        self.console_frame.grid(row=2, column=0, columnspan=2, sticky="ew", padx=3, pady=(0, 3))

    def _create_palette(self):
        self.palette_frame.configure(padding=6)
        self._palette_listbox = tk.Listbox(self.palette_frame,
                                           bg=Theme.BG_PANEL,
                                           fg=Theme.TEXT,
                                           selectbackground=Theme.ACCENT,
                                           selectforeground="white",
                                           selectborderwidth=0,
                                           activestyle="none",
                                           font=("Segoe UI", 9),
                                           highlightthickness=0,
                                           border=0)
        self._palette_scroll = ttk.Scrollbar(self.palette_frame, orient=tk.VERTICAL)
        self._palette_listbox.configure(yscrollcommand=self._palette_scroll.set)
        self._palette_scroll.configure(command=self._palette_listbox.yview)

        self._palette_listbox.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        self._palette_scroll.pack(side=tk.RIGHT, fill=tk.Y)
        self._palette_listbox.bind("<<ListboxSelect>>", self._on_palette_select)

        self._palette_categories = {}
        self._palette_items = []
        for name, cls in sorted(NodeRegistry.get_all_nodes().items()):
            cat = getattr(cls, 'category', 'General')
            if cat not in self._palette_categories:
                self._palette_categories[cat] = []
            self._palette_categories[cat].append(name)

    def _populate_palette(self):
        self._palette_listbox.delete(0, tk.END)
        self._palette_items.clear()
        for cat in sorted(self._palette_categories.keys()):
            cat_color = Theme.CAT_COLORS.get(cat, Theme.TEXT)
            # Category header
            idx = self._palette_listbox.size()
            self._palette_listbox.insert(tk.END, f"-- {cat} --")
            self._palette_listbox.itemconfig(idx, fg=cat_color)
            self._palette_items.append(("header", cat))
            for name in self._palette_categories[cat]:
                self._palette_listbox.insert(tk.END, name)
                self._palette_items.append(("node", name))

    def _create_canvas(self):
        # Canvas wrapper with scrollbar
        self.canvas_scroll_frame = ttk.Frame(self.canvas_wrapper)
        self.canvas_scroll_frame.pack(fill=tk.BOTH, expand=True)

        self.graph_canvas = GraphCanvas(self.canvas_scroll_frame)
        self.graph_canvas.pack(fill=tk.BOTH, expand=True, side=tk.LEFT)

        self.graph_canvas_y_scroll = ttk.Scrollbar(self.canvas_wrapper, orient=tk.VERTICAL,
                                                   command=self.graph_canvas.yview)
        self.graph_canvas_x_scroll = ttk.Scrollbar(self.canvas_wrapper, orient=tk.HORIZONTAL,
                                                    command=self.graph_canvas.xview)
        self.graph_canvas.configure(yscrollcommand=self.graph_canvas_y_scroll.set,
                                     xscrollcommand=self.graph_canvas_x_scroll.set)
        self.graph_canvas_y_scroll.pack(side=tk.RIGHT, fill=tk.Y)
        self.graph_canvas_x_scroll.pack(side=tk.BOTTOM, fill=tk.X)

    def _create_properties(self):
        self.props_frame.configure(padding=8)
        self._prop_title = ttk.Label(self.props_frame, text="No node selected",
                                      foreground=Theme.TEXT_DIM, font=("Segoe UI", 9))
        self._prop_title.pack(anchor="w", pady=(0, 8))

        self._prop_scroll = ttk.Scrollbar(self.props_frame)
        self._prop_scroll.pack(side=tk.RIGHT, fill=tk.Y)

        self._prop_text = tk.Text(self.props_frame, bg=Theme.BG_INPUT,
                                  fg=Theme.TEXT, font=("Consolas", 9),
                                  yscrollcommand=self._prop_scroll.set,
                                  state=tk.DISABLED,
                                  highlightthickness=0,
                                  padx=4, pady=4)
        self._prop_text.pack(fill=tk.BOTH, expand=True)
        self._prop_scroll.config(command=self._prop_text.yview)

    def _create_console(self):
        self.console_frame.configure(padding=5)
        self.console_frame.configure(height=120)
        self._console_text = tk.Text(self.console_frame, bg="#0d0e14",
                                      fg=Theme.GREEN, font=("Consolas", 9),
                                      height=5, state=tk.DISABLED,
                                      insertbackground=Theme.TEXT,
                                      highlightthickness=0,
                                      padx=4, pady=2)
        self._console_text.pack(fill=tk.BOTH, expand=True)

        ttk.Button(self.console_frame, text="Clear", command=self._clear_console,
                  width=10).pack(anchor="e", pady=(3, 0))

    def _load_plugins(self):
        nodes_dir = os.path.join(os.path.dirname(__file__), '..', 'nodes')
        self._import_nodes_dir(nodes_dir)

        plugins_dir = os.path.join(os.path.dirname(__file__), '..', 'plugins')
        try:
            NodeRegistry.load_plugins(plugins_dir)
            self._log(f"Plugins: {plugins_dir}", "dim")
        except Exception as e:
            self._log(f"Plugin load: {e}", "dim")

        # Rebuild category list from freshly loaded registry
        self._palette_categories = {}
        for name, cls in sorted(NodeRegistry.get_all_nodes().items()):
            cat = getattr(cls, 'category', 'General')
            if cat not in self._palette_categories:
                self._palette_categories[cat] = []
            self._palette_categories[cat].append(name)
        self._populate_palette()
        self._log(f"  {len(NodeRegistry.get_all_nodes())} node types", "green")

    def _import_nodes_dir(self, nodes_dir):
        if not os.path.isdir(nodes_dir):
            return
        for root, dirs, files in os.walk(nodes_dir):
            dirs[:] = [d for d in dirs if d != '__pycache__']
            for f in files:
                if f.endswith('.py') and not f.startswith('__'):
                    filepath = os.path.join(root, f)
                    try:
                        spec = importlib.util.spec_from_file_location(
                            f"nodes.{f[:-3]}", filepath)
                        if spec and spec.loader:
                            mod = importlib.util.module_from_spec(spec)
                            spec.loader.exec_module(mod)
                    except Exception:
                        pass

    # ===================== Actions =====================

    def on_connect_ports(self, src_id, src_port, dst_id, dst_port):
        try:
            self.engine.connect(src_id, src_port, dst_id, dst_port)
            self.graph_canvas.connect_ports(src_id, src_port, dst_id, dst_port)
            self._log(f"  {src_id}:{src_port} -> {dst_id}:{dst_port}", "green")
        except Exception as e:
            self._log(f"Connection error: {e}", "red")

    def on_delete_node(self, node_id):
        self._delete_selected()

    def on_node_double_click(self, node_obj):
        pass

    def _new_graph(self):
        self.engine = Engine()
        self.graph_canvas.nodes.clear()
        self.graph_canvas.connections.clear()
        self.graph_canvas.delete("all")
        self.graph_canvas.delete("port")
        self.graph_canvas.delete("connection")
        self.graph_canvas.delete("shadow")
        self.node_counter = 0
        self._palette_listbox.selection_clear(0, tk.END)
        self._file_path = None
        self._log("New graph", "green")

    def _open_dialog(self):
        path = filedialog.askopenfilename(
            title="Open Graph",
            filetypes=[("JSON files", "*.json"), ("All files", "*.*")])
        if path:
            self._load_graph(path)

    def _load_graph(self, filepath):
        try:
            with open(filepath, 'r') as f:
                data = json.load(f)
            self.engine = Engine.from_dict({
                "nodes": data.get("nodes", []),
                "edges": data.get("edges", [])})
            self._file_path = filepath
            self._log(f"Loaded: {os.path.basename(filepath)}", "green")
            self._log(f"  {len(self.engine.list_nodes())} nodes, "
                      f"{len(self.engine.graph._edges)} connections", "green")
            self._redraw_canvas()
        except Exception as e:
            self._log(f"Load error: {e}", "red")

    def _save_graph(self):
        path = self._file_path or filedialog.asksaveasfilename(
            title="Save Graph",
            defaultextension=".json",
            filetypes=[("JSON files", "*.json")])
        if path:
            self._do_save(path)

    def _save_as_dialog(self):
        path = filedialog.asksaveasfilename(
            title="Save Graph As",
            defaultextension=".json",
            filetypes=[("JSON files", "*.json")])
        if path:
            self._do_save(path)

    def _do_save(self, filepath):
        try:
            data = self.engine.to_dict()
            os.makedirs(os.path.dirname(filepath) or ".", exist_ok=True)
            with open(filepath, 'w') as f:
                json.dump(data, f, indent=2)
            self._file_path = filepath
            self._log(f"Saved: {os.path.basename(filepath)}", "green")
        except Exception as e:
            self._log(f"Save error: {e}", "red")

    def _delete_selected(self):
        if self.graph_canvas.selected_node:
            nid = self.graph_canvas.selected_node.node_obj.id
            self.engine.remove_node(nid)
            self.graph_canvas.remove_node(nid)
            self.graph_canvas.deselect_all()
            self._log(f"Deleted: {nid}")
            self._update_properties()

    def _start_audio(self):
        try:
            self.engine.start()
            self._audio_status.config(text="● Running", foreground=Theme.GREEN)
            self._log("Audio started", "green")
        except Exception as e:
            self._log(f"Audio error: {e}", "red")

    def _stop_audio(self):
        if self.engine.is_running:
            self.engine.stop()
            self._audio_status.config(text="● Stopped", foreground=Theme.TEXT_DIM)
            self._log("Audio stopped", "green")

    def _toggle_audio(self):
        if self.engine.is_running:
            self._stop_audio()
        else:
            self._start_audio()

    def _export_wav(self):
        try:
            self._log("Exporting WAV...", "yellow")
            import wave
            import time
            sr = 44100
            dur = 2.0
            n = int(sr * dur)
            samples = np.zeros(n, dtype=np.float32)
            bs = 64
            recorded = 0
            self.engine.start()
            while recorded < n and self.engine.is_running:
                try:
                    results = self.engine.graph.process_block(bs)
                    for nid, nout in results.items():
                        if nid in self.engine.graph.nodes:
                            nd = self.engine.graph.nodes[nid]
                            for pn, pt in nd.outputs.items():
                                sig = pt.value
                                if hasattr(sig, '__len__') and len(sig) > 0:
                                    s = recorded
                                    e = min(s + len(sig), n)
                                    if e > s:
                                        samples[s:e] += sig[:e-s]
                    recorded = min(recorded + bs, n)
                except Exception:
                    pass
                time.sleep(0.001)
            self.engine.stop()

            path = "presets/export.wav"
            os.makedirs("presets", exist_ok=True)
            with wave.open(path, 'wb') as wf:
                wf.setnchannels(1)
                wf.setsampwidth(2)
                wf.setframerate(sr)
                samples = np.clip(samples, -1.0, 1.0)
                wf.writeframes(np.int16(samples * 32767).tobytes())
            self._log(f"Exported: presets/export.wav", "green")
        except Exception as e:
            self._log(f"Export error: {e}", "red")

    def _toggle_palette(self):
        if self.palette_frame.winfo_ismapped():
            self.palette_frame.pack_forget()
            self._palette_visible = False
        else:
            self.palette_frame.pack(fill=tk.X, padx=0, pady=0)
            self._palette_visible = True

    def _toggle_console(self):
        h = self.console_frame.winfo_height()
        if h < 50:
            self.console_frame.configure(height=120)
            self._console_visible = True
        else:
            self.console_frame.configure(height=25)
            self._console_visible = False

    def _redraw_canvas(self):
        self.graph_canvas.nodes.clear()
        self.graph_canvas.connections.clear()
        self.graph_canvas.delete("all")

        for nid, node in self.engine.graph.nodes.items():
            self.graph_canvas.add_node(node, 200 + len(self.graph_canvas.nodes) * 30,
                                       150 + len(self.graph_canvas.nodes) * 40)

        for src_id, dst_id in self.engine.graph._edges:
            if src_id in self.engine.graph.nodes and dst_id in self.engine.graph.nodes:
                src_node = self.engine.graph.nodes[src_id]
                dst_node = self.engine.graph.nodes[dst_id]
                src_ports = list(src_node.outputs.keys())
                dst_ports = list(dst_node.inputs.keys())
                if src_ports and dst_ports:
                    self.graph_canvas.connect_ports(
                        src_id, src_ports[0], dst_id, dst_ports[0])

    def _on_palette_select(self, event):
        selection = self._palette_listbox.curselection()
        if not selection:
            return
        node_name = self._palette_listbox.get(selection[0])
        # Skip category headers
        if node_name.startswith("──"):
            return
        try:
            cls = NodeRegistry.get_node_class(node_name)
            nid = f"{node_name.lower()}_{self.node_counter:03d}"
            self.node_counter += 1
            node = cls(node_id=nid)
            node_id = self.engine.add_node(node)

            x, y = 400, 200
            if self.graph_canvas.nodes:
                last_node = list(self.graph_canvas.nodes.values())[-1]
                x = last_node.x + Theme.NODE_W + 40
                y = last_node.y

            self.graph_canvas.add_node(node, x, y)
            self._log(f"Added: {node_name} as '{nid}'", "green")
        except Exception as e:
            self._log(f"Error: {e}", "red")

    def _update_properties(self):
        if self.graph_canvas.selected_node:
            node = self.graph_canvas.selected_node.node_obj
            cat = getattr(node, 'category', 'General')
            cat_color = Theme.CAT_COLORS.get(cat, Theme.TEXT)
            self._prop_title.config(text=f"{node.id}  •  {cat}",
                                     foreground=cat_color)

            params_text = f"Type: {node.__class__.__name__}\n\n"
            for pname, pval in sorted(node.params.items()):
                if pname.startswith("_"):
                    continue
                if isinstance(pval, float):
                    params_text += f"  {pname}: {pval:.4f}\n"
                else:
                    params_text += f"  {pname}: {pval}\n"

            self._prop_text.config(state=tk.NORMAL)
            self._prop_text.delete(1.0, tk.END)
            self._prop_text.insert(tk.END, params_text)
            self._prop_text.config(state=tk.DISABLED)

    def _process_console_queue(self):
        while not self._console_queue.empty():
            msg, color = self._console_queue.get()
            self._console_text.config(state=tk.NORMAL)
            self._console_text.insert(tk.END, msg, color)
            self._console_text.see(tk.END)
            self._console_text.config(state=tk.DISABLED)
        self.root.after(100, self._process_console_queue)

    def _log(self, msg, color="white"):
        self._log_queue_item(msg, color)

    def _log_queue_item(self, msg, color):
        tag = color if color in ("green", "red", "yellow", "dim") else "white"
        self._console_text.config(state=tk.NORMAL)
        self._console_text.insert(tk.END, msg + "\n", tag)
        self._console_text.see(tk.END)
        self._console_text.config(state=tk.DISABLED)

    def _clear_console(self):
        self._console_text.config(state=tk.NORMAL)
        self._console_text.delete(1.0, tk.END)
        self._console_text.config(state=tk.DISABLED)


def main():
    root = tk.Tk()
    RetroApp(root)
    root.mainloop()


if __name__ == "__main__":
    main()
