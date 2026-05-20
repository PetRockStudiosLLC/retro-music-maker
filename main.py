"""Retro Music Maker - Main entry point.

Usage:
    python main.py              Show help
    python main.py api          Start FastAPI server (port 8000)
    python main.py gui          Start Tkinter GUI (legacy)
    python main.py cli          Start interactive CLI
    python main.py preset <f>   Play a saved preset
    python main.py midi-list    List available MIDI ports
    npm run dev                 Start React dev server (port 5173)
"""
import sys
import os


def main():
    if len(sys.argv) > 1:
        cmd = sys.argv[1]

        if cmd == "cli":
            from ui.cli import main as cli_main

            cli_main()
            return

        elif cmd == "gui":
            from ui.app import main as gui_main

            gui_main()
            return

        elif cmd == "api":
            import uvicorn

            uvicorn.run("api.server:app", host="0.0.0.0", port=8000, reload=True)
            return

        elif cmd == "preset":
            from core.engine import Engine
            import json

            preset_file = sys.argv[2] if len(sys.argv) > 2 else None
            if not preset_file:
                print("Usage: python main.py preset <file.json>")
                return
            with open(preset_file, "r") as f:
                data = json.load(f)
            engine = Engine.from_dict(data)
            print(f"Loaded preset with {len(engine.list_nodes())} nodes")
            engine.start()
            print("Playing... Press Ctrl+C to stop.")
            try:
                import time

                while True:
                    time.sleep(1)
            except KeyboardInterrupt:
                engine.stop()
                print("Stopped.")
            return

        elif cmd == "midi-list":
            try:
                from ui.midi_input import MIDIInput

                ports = MIDIInput.list_ports()
                if ports:
                    print("Available MIDI ports:")
                    for i, p in enumerate(ports):
                        print(f"  [{i}] {p}")
                else:
                    print("No MIDI inputs found.")
            except Exception as e:
                print(f"Error: {e}")
            return

    print(
        """
Retro Music Maker - Node-based retro music creation
==================================================

Usage:
    python main.py api              Start API server (port 8000) + React frontend
    python main.py cli              Start interactive CLI
    python main.py gui              Start Tkinter GUI (legacy)
    python main.py preset <file>    Play a saved preset
    python main.py midi-list        List MIDI input ports

    npm run dev                     Start React dev server (port 5173)
                                    Requires: npm install in frontend/

Node Types:
"""
    )
    try:
        from core import NodeRegistry
        import importlib.util

        nodes_dir = os.path.join(
            os.path.dirname(os.path.abspath(__file__)), "nodes"
        )
        if os.path.isdir(nodes_dir):
            for root, dirs, files in os.walk(nodes_dir):
                for f in files:
                    if f.endswith(".py") and not f.startswith("__"):
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
        nodes = NodeRegistry.get_all_nodes()
        cats = {}
        for name, cls in nodes.items():
            cat = getattr(cls, "category", "General")
            cats.setdefault(cat, []).append(name)
        for cat in sorted(cats.keys()):
            names = ", ".join(sorted(cats[cat]))
            print(f"  {cat}: {names}")
    except Exception:
        pass

    print(
        """
Plugin Development:
    Create Python files in the 'plugins/' directory
    Inherit from core.node.Node, decorate with @NodeRegistry.register

See python_api/examples/my_first_node.py for a template.
"""
    )


if __name__ == "__main__":
    main()
