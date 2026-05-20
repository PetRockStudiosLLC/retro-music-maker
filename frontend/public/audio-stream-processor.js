class AudioStreamProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this.queue = [];
    this.port.onmessage = (e) => {
      if (e.data.type === "chunk") {
        this.queue.push(e.data.data);
      }
    };
  }

  process(inputs, outputs) {
    const output = outputs[0][0];
    if (this.queue.length > 0) {
      const chunk = this.queue.shift();
      for (let i = 0; i < output.length; i++) {
        output[i] = i < chunk.length ? chunk[i] : 0;
      }
    } else {
      output.fill(0);
    }
    return true;
  }
}

registerProcessor("audio-stream-processor", AudioStreamProcessor);
