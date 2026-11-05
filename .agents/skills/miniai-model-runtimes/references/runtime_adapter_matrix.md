# Modality Runtime Adapter Matrix

| Modality | Enum Variant | Default CPU Backend | Optional Accelerators | Artifact Formats |
| :--- | :--- | :--- | :--- | :--- |
| LLM Text | `Modality::Llm` | CPU (GGUF / Candle / ONNX) | CUDA, Metal, Vulkan, DirectML, ROCm | `.gguf`, `.safetensors`, `.onnx` |
| Vision Understanding | `Modality::Vision` | CPU (ONNX / TorchScript / GGUF-MMProj) | CUDA, Metal, Vulkan, DirectML, CoreML | `.onnx`, `.safetensors`, `.gguf` |
| Image Generation | `Modality::ImageGeneration` | CPU (Latent Diffusion ONNX / Candle) | CUDA, Metal, Vulkan, DirectML | `.safetensors`, `.onnx` |
| Speech ASR | `Modality::SpeechAsr` | CPU (Whisper / Conformer ONNX/GGUF) | CUDA, Metal, Vulkan, CoreML | `.bin`, `.gguf`, `.onnx` |
| Speech TTS | `Modality::SpeechTts` | CPU (Piper / VITS / Kokoro ONNX) | CUDA, Metal, CoreML | `.onnx`, `.safetensors` |
| Music Generation | `Modality::Music` | CPU (MusicGen / AudioCraft ONNX) | CUDA, Metal, Vulkan | `.safetensors`, `.onnx` |
| Video Understanding & Gen | `Modality::Video` | CPU (Frame-sampled Vision + Temporal ONNX) | CUDA, Metal, Vulkan | `.safetensors`, `.onnx` |

