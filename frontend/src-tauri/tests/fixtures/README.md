`audio-source-speech.wav` is a deterministic 3-second, 16 kHz mono PCM fixture
generated locally with macOS Samantha text-to-speech. It contains the beginning
of: “Each transcript passage keeps its capture source. The microphone and system
audio remain separate.” It contains no recorded participant audio.

Routing tests inject the same fixture into known microphone/system channels,
with silence for single-channel, alternating, and simultaneous input scenarios.
The checked-in WAV lets the native VAD tests run on either platform without a
text-to-speech service or model download.

`echo-near-end-speech.wav` is a 16 kHz mono PCM fixture generated locally with
macOS Daniel at 165 words per minute: “I am speaking into the microphone. Please
keep my own voice while the speakers play.” Echo tests mix this independent
voice with delayed, attenuated playback. It contains no participant recording.
