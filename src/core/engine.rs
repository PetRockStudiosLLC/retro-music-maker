use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use crate::core::{BlockSize, Graph, Node, ParamValue, NodeInfo, graph::GraphState};

type SharedGraph = Arc<parking_lot::Mutex<Graph>>;

pub struct Engine {
    graph: SharedGraph,
    pub sample_rate: u32,
    pub block_size: BlockSize,
    running: Arc<AtomicBool>,
    node_positions: HashMap<String, [f64; 2]>,
    audio_handle: Option<thread::JoinHandle<()>>,
}

impl Engine {
    pub fn new(sample_rate: u32, block_size: BlockSize) -> Self {
        Self {
            graph: Arc::new(parking_lot::Mutex::new(Graph::new())),
            sample_rate,
            block_size,
            running: Arc::new(AtomicBool::new(false)),
            node_positions: HashMap::new(),
            audio_handle: None,
        }
    }

    pub fn add_node(&mut self, node: Box<dyn Node>, position: [f64; 2]) -> String {
        let id = node.id().to_string();
        self.node_positions.insert(id.clone(), position);
        self.graph.lock().add_node(node);
        id
    }

    pub fn remove_node(&mut self, node_id: &str) -> bool {
        self.node_positions.remove(node_id);
        self.graph.lock().remove_node(node_id)
    }

    pub fn add_edge(&mut self, source: String, source_handle: String, target: String, target_handle: String) {
        use crate::core::graph::Edge;
        self.graph.lock().add_edge(Edge {
            source,
            source_handle,
            target,
            target_handle,
        });
    }

    pub fn remove_edge(&mut self, source: &str, target: &str) {
        self.graph.lock().remove_edge(source, target);
    }

    pub fn set_node_position(&mut self, node_id: &str, position: [f64; 2]) {
        self.node_positions.insert(node_id.to_string(), position);
    }

    pub fn set_param(&mut self, node_id: &str, param_name: &str, value: ParamValue) {
        let mut g = self.graph.lock();
        if let Some(node) = g.get_node_mut(node_id) {
            node.set_param(param_name, value);
        }
    }

    pub fn set_node_param(&mut self, node_id: &str, param_name: &str, value: ParamValue) {
        self.set_param(node_id, param_name, value);
    }

    pub fn get_node_info(&self, node_id: &str) -> Option<NodeInfo> {
        let pos = self.node_positions.get(node_id).copied().unwrap_or([0.0, 0.0]);
        let g = self.graph.lock();
        g.get_node(node_id).map(|n| n.to_info(pos))
    }

    pub fn get_graph_state(&self) -> GraphState {
        let g = self.graph.lock();
        g.to_state(&self.node_positions)
    }

    pub fn node_count(&self) -> usize {
        self.graph.lock().node_ids().len()
    }

    pub fn start(&mut self) -> Result<(), String> {
        if self.running.load(Ordering::Relaxed) {
            return Ok(());
        }

        self.stop_internal();

        let host = cpal::default_host();
        let device = host.default_output_device().ok_or("No default output device")?;

        let config = device.default_output_config().map_err(|e| e.to_string())?;
        let channels = config.channels() as usize;
        let sample_format = config.sample_format();

        let graph_arc = Arc::clone(&self.graph);
        let running_arc = Arc::clone(&self.running);

        self.running.store(true, Ordering::Relaxed);

        let handle = thread::spawn(move || {
            let error_handler = |err: cpal::StreamError| {
                eprintln!("Audio stream error: {}", err);
            };

            let stream = match sample_format {
                cpal::SampleFormat::F32 => {
                    let cb_graph = Arc::clone(&graph_arc);
                    let cb_running = Arc::clone(&running_arc);
                    let data = move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        if !cb_running.load(Ordering::Relaxed) {
                            return;
                        }
                        let frame_count = output.len() / channels;
                        let mut g = cb_graph.lock();
                        if let Ok(buffers) = g.process_block(frame_count) {
                            let output_ids: Vec<String> = g.nodes.iter()
                                .filter(|(_, n)| n.name() == "AudioOutput")
                                .map(|(id, _)| id.clone())
                                .collect();
                            let active: Vec<_> = buffers.iter()
                                .filter(|(id, _)| output_ids.contains(id))
                                .filter_map(|(_, b)| if b.is_empty() { None } else { Some(b) })
                                .collect();
                            let count = if active.is_empty() { 1.0f32 } else { active.len() as f32 };
                            for i in 0..frame_count {
                                let mut s = 0.0f32;
                                for buf in &active {
                                    s += buf.get(i).copied().unwrap_or(0.0);
                                }
                                if count > 0.0 { s /= count; }
                                s = s.min(1.0).max(-1.0);
                                output[i * channels] = s;
                                if channels > 1 && i * channels + 1 < output.len() {
                                    output[i * channels + 1] = s;
                                }
                            }
                        }
                    };
                    device.build_output_stream(&config.into(), data, error_handler, None).ok()
                }
                cpal::SampleFormat::I16 => {
                    let cb_graph = Arc::clone(&graph_arc);
                    let cb_running = Arc::clone(&running_arc);
                    let data = move |output: &mut [i16], _: &cpal::OutputCallbackInfo| {
                        if !cb_running.load(Ordering::Relaxed) {
                            return;
                        }
                        let frame_count = output.len() / channels;
                        let mut g = cb_graph.lock();
                        if let Ok(buffers) = g.process_block(frame_count) {
                            let output_ids: Vec<String> = g.nodes.iter()
                                .filter(|(_, n)| n.name() == "AudioOutput")
                                .map(|(id, _)| id.clone())
                                .collect();
                            let active: Vec<_> = buffers.iter()
                                .filter(|(id, _)| output_ids.contains(id))
                                .filter_map(|(_, b)| if b.is_empty() { None } else { Some(b) })
                                .collect();
                            let count = if active.is_empty() { 1.0f32 } else { active.len() as f32 };
                            for i in 0..frame_count {
                                let mut s = 0.0f32;
                                for buf in &active {
                                    s += buf.get(i).copied().unwrap_or(0.0);
                                }
                                if count > 0.0 { s /= count; }
                                s = s.min(1.0).max(-1.0);
                                let sample = (s * 32767.0) as i16;
                                output[i * channels] = sample;
                                if channels > 1 && i * channels + 1 < output.len() {
                                    output[i * channels + 1] = sample;
                                }
                            }
                        }
                    };
                    device.build_output_stream(&config.into(), data, error_handler, None).ok()
                }
                _ => {
                    eprintln!("Unsupported sample format: {:?}", sample_format);
                    None
                }
            };

            if let Some(stream) = stream {
                if let Err(e) = stream.play() {
                    eprintln!("Failed to play stream: {}", e);
                }
                while running_arc.load(Ordering::Relaxed) {
                    thread::sleep(std::time::Duration::from_millis(100));
                }
            } else {
                running_arc.store(false, Ordering::Relaxed);
            }
        });

        self.audio_handle = Some(handle);
        Ok(())
    }

 
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        self.stop_internal();
    }

    fn stop_internal(&mut self) {
        if let Some(handle) = self.audio_handle.take() {
            drop(handle);
        }
        self.running.store(false, Ordering::Relaxed);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn export_wav(&self, duration: f64, output_sample_rate: Option<u32>) -> Result<Vec<u8>, String> {
        let total_samples = (duration * self.sample_rate as f64) as usize;
        let block_size = self.block_size;
        let mut all_samples: Vec<f32> = Vec::with_capacity(total_samples);

        let mut g = self.graph.lock();
        let mut remaining = total_samples;
        while remaining > 0 {
            let bs = (block_size as usize).min(remaining);
            if let Ok(buffers) = g.process_block(bs) {
                let output_ids: Vec<String> = g.nodes.iter()
                    .filter(|(_, n)| n.name() == "AudioOutput")
                    .map(|(id, _)| id.clone())
                    .collect();
                let active: Vec<_> = buffers.iter()
                    .filter(|(id, _)| output_ids.contains(id))
                    .filter_map(|(_, b)| if b.is_empty() { None } else { Some(b) })
                    .collect();
                let count = if active.is_empty() { 1.0f32 } else { active.len() as f32 };
                for i in 0..bs {
                    let mut s = 0.0f32;
                    for buf in &active {
                        s += buf.get(i).copied().unwrap_or(0.0);
                    }
                    if count > 0.0 { s /= count; }
                    s = s.min(1.0).max(-1.0);
                    all_samples.push(s);
                }
            }
            remaining -= bs;
        }

        let resampled = if let Some(target_rate) = output_sample_rate {
            if target_rate != self.sample_rate {
                linear_resample(&all_samples, self.sample_rate, target_rate)
            } else {
                all_samples
            }
        } else {
            all_samples
        };

        let final_rate = output_sample_rate.unwrap_or(self.sample_rate);
        let nchannels = 1u16;
        let bits_per_sample = 16u16;
        let byte_rate = final_rate * nchannels as u32 * bits_per_sample as u32 / 8;
        let block_align = nchannels * bits_per_sample / 8;
        let data_size = (resampled.len() * bits_per_sample as usize / 8) as u32;
        let chunk_size = 4u32 + (8 + 24 + data_size) as u32;

        let mut wav = Vec::with_capacity(44 + resampled.len() * 2);
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&chunk_size.to_le_bytes());
        wav.extend_from_slice(b"WAVE");
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&nchannels.to_le_bytes());
        wav.extend_from_slice(&final_rate.to_le_bytes());
        wav.extend_from_slice(&byte_rate.to_le_bytes());
        wav.extend_from_slice(&block_align.to_le_bytes());
        wav.extend_from_slice(&bits_per_sample.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&data_size.to_le_bytes());

        for sample in &resampled {
            let pcm = (*sample * 32767.0) as i16;
            wav.extend_from_slice(&pcm.to_le_bytes());
        }

        Ok(wav)
    }

    pub fn clear(&mut self) {
        self.stop();
        self.graph.lock().clear();
        self.node_positions.clear();
    }

    pub fn get_file_output_buffer(&self, node_id: &str) -> Result<Vec<f32>, String> {
        let g = self.graph.lock();
        let node = g.get_node(node_id).ok_or(format!("Node {} not found", node_id))?;
        node.get_recording().ok_or("Node does not support recording".to_string())
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new(44100, 512)
    }
}

fn linear_resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate {
        return samples.to_vec();
    }
    let ratio = from_rate as f64 / to_rate as f64;
    let out_len = (samples.len() as f64 / ratio) as usize;
    let mut result = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let pos = i as f64 * ratio;
        let idx = pos.floor() as usize;
        let frac = (pos - idx as f64) as f32;
        let next = (idx + 1).min(samples.len() - 1);
        let val = samples[idx] * (1.0 - frac) + samples[next] * frac;
        result.push(val);
    }
    result
}
