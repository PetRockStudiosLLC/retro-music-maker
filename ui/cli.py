"""CLI interface for Retro Music Maker."""
import sys
import json
import cmd
import numpy as np

# Fix readline compatibility on Windows
if sys.platform == 'win32':
    import readline
    if hasattr(readline, 'backend'):
        delattr(readline, 'backend')

from core.engine import Engine
from core import NodeRegistry


class RetroCLI(cmd.Cmd):
    """Interactive CLI for Retro Music Maker."""

    intro = '\nWelcome to Retro Music Maker! Type help or ? to list commands.\n'
    prompt = '\nrm> '

    def __init__(self):
        import sys
        # Disable readline on Windows for compatibility
        if sys.platform == 'win32':
            self.use_rawinput = False
        super().__init__()
        self.engine = None
        self._setup_node_registry()

    def _setup_node_registry(self):
        """Load all node types."""
        from core import NodeRegistry
        # Auto-load from nodes directory
        import os
        import importlib.util

        nodes_dir = os.path.join(os.path.dirname(os.path.dirname(__file__)), 'nodes')
        if os.path.isdir(nodes_dir):
            for root, dirs, files in os.walk(nodes_dir):
                for f in files:
                    if f.endswith('.py') and not f.startswith('__'):
                        filepath = os.path.join(root, f)
                        try:
                            spec = importlib.util.spec_from_file_location(
                                f"nodes.{f[:-3]}", filepath
                            )
                            if spec and spec.loader:
                                module = importlib.util.module_from_spec(spec)
                                spec.loader.exec_module(module)
                        except Exception:
                            pass

    def do_load(self, arg):
        """Load a node graph from a JSON file."""
        try:
            with open(arg, 'r') as f:
                data = json.load(f)
            self.engine = Engine.from_dict(data)
            print(f"Loaded {len(self.engine.list_nodes())} nodes")
        except FileNotFoundError:
            print(f"File not found: {arg}")
        except Exception as e:
            print(f"Error loading graph: {e}")

    def do_save(self, arg):
        """Save the current node graph to a JSON file."""
        if not self.engine:
            print("No graph loaded. Use 'new' first.")
            return
        try:
            with open(arg, 'w') as f:
                json.dump(self.engine.to_dict(), f, indent=2)
            print(f"Saved graph to {arg}")
        except Exception as e:
            print(f"Error saving graph: {e}")

    def do_new(self, arg):
        """Create a new empty graph."""
        self.engine = Engine()
        print("Created new empty graph")

    def do_add(self, arg):
        """Add a node to the graph. Usage: add <node_type> [<id>]"""
        if not self.engine:
            print("No graph. Create one first with 'new'")
            return
        parts = arg.split()
        if len(parts) < 1:
            print("Usage: add <node_type> [<id>]")
            return
        node_type = parts[0]
        node_id = parts[1] if len(parts) > 1 else None

        try:
            node_class = NodeRegistry.get_node_class(node_type)
            node = node_class(node_id=node_id)
            self.engine.add_node(node)
            print(f"Added {node_type} as '{node.id}'")
        except KeyError:
            print(f"Unknown node type: {node_type}")
            print(f"Available: {list(NodeRegistry.get_all_nodes().keys())}")
        except Exception as e:
            print(f"Error adding node: {e}")

    def do_connect(self, arg):
        """Connect two nodes. Usage: connect <src_id>:<port> <dst_id>:<port>"""
        if not self.engine:
            print("No graph.")
            return
        parts = arg.split()
        if len(parts) < 2:
            print("Usage: connect <src_id>:<port> <dst_id>:<port>")
            return
        src = parts[0].split(':')
        dst = parts[1].split(':')
        if len(src) != 2 or len(dst) != 2:
            print("Usage: connect <src_id>:<port> <dst_id>:<port>")
            return
        try:
            self.engine.connect(src[0], src[1], dst[0], dst[1])
            print(f"Connected {src[0]}:{src[1]} -> {dst[0]}:{dst[1]}")
        except Exception as e:
            print(f"Error connecting: {e}")

    def do_disconnect(self, arg):
        """Disconnect two nodes. Usage: disconnect <src_id>:<port> <dst_id>:<port>"""
        if not self.engine:
            print("No graph.")
            return
        parts = arg.split()
        if len(parts) < 2:
            print("Usage: disconnect <src_id>:<port> <dst_id>:<port>")
            return
        src = parts[0].split(':')
        dst = parts[1].split(':')
        try:
            self.engine.disconnect(src[0], src[1], dst[0], dst[1])
            print(f"Disconnected {src[0]}:{src[1]} -> {dst[0]}:{dst[1]}")
        except Exception as e:
            print(f"Error disconnecting: {e}")

    def do_start(self, arg):
        """Start audio playback."""
        if not self.engine:
            print("No graph loaded.")
            return
        self.engine.start()
        print("Audio started. Press Ctrl+C to stop.")

    def do_stop(self, arg):
        """Stop audio playback."""
        if self.engine and self.engine.is_running:
            self.engine.stop()
            print("Audio stopped.")

    def do_nodes(self, arg):
        """List all nodes in the graph."""
        if not self.engine:
            print("No graph.")
            return
        nodes = self.engine.list_nodes()
        if not nodes:
            print("No nodes in graph.")
        else:
            for nid in nodes:
                node = self.engine.get_node(nid)
                print(f"  {nid}: {node.__class__.__name__}")

    def do_params(self, arg):
        """Set a node parameter. Usage: params <node_id> <param> <value>"""
        if not self.engine:
            print("No graph.")
            return
        parts = arg.split()
        if len(parts) < 3:
            print("Usage: params <node_id> <param> <value>")
            return
        node_id = parts[0]
        param = parts[1]
        value = float(parts[2]) if '.' in parts[2] else int(parts[2])

        node = self.engine.get_node(node_id)
        if node:
            node.set_param(param, value)
            print(f"Set {node_id}.{param} = {value}")
        else:
            print(f"Node not found: {node_id}")

    def do_info(self, arg):
        """Show information about available node types."""
        nodes = NodeRegistry.get_all_nodes()
        print(f"\nAvailable node types ({len(nodes)}):")
        categories = {}
        for name, cls in nodes.items():
            cat = getattr(cls, 'category', 'General')
            categories.setdefault(cat, []).append(name)
        for cat, names in sorted(categories.items()):
            print(f"\n  [{cat}]")
            for name in sorted(names):
                cls = nodes[name]
                desc = getattr(cls, 'description', '')
                print(f"    {name:20s} {desc}")

    def do_devices(self, arg):
        """List available audio devices."""
        if self.engine:
            self.engine.backend.list_devices()

    def do_quit(self, arg):
        """Quit the CLI."""
        if self.engine and self.engine.is_running:
            self.engine.stop()
        return True

    do_exit = do_quit
    do_q = do_quit

  

    def emptyline(self):
        """Do nothing on empty line."""
        pass

    def default(self, line):
        """Handle unknown commands."""
        print(f"Unknown command: {line}")
        print("Type 'help' for available commands.")


def main():
    """Start the Retro Music Maker CLI."""
    cli = RetroCLI()
    try:
        cli.cmdloop()
    except KeyboardInterrupt:
        if cli.engine and cli.engine.is_running:
            cli.engine.stop()
        print("\nGoodbye!")


if __name__ == '__main__':
    main()
