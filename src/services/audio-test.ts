// Simple audio test to verify browser can play audio
export function testAudioPlayback() {
  console.log('Testing basic audio playback...')
  
  // Create a simple tone using Web Audio API
  const audioContext = new (window.AudioContext || (window as any).webkitAudioContext)()
  const oscillator = audioContext.createOscillator()
  const gainNode = audioContext.createGain()
  
  oscillator.connect(gainNode)
  gainNode.connect(audioContext.destination)
  
  oscillator.frequency.value = 440 // A4 note
  gainNode.gain.value = 0.1 // Low volume
  
  oscillator.start()
  
  // Stop after 1 second
  setTimeout(() => {
    oscillator.stop()
    console.log('Audio test complete - you should have heard a beep')
  }, 1000)
  
  // Also test HTML audio element
  const audio = new Audio()
  audio.src = 'data:audio/wav;base64,UklGRnoGAABXQVZFZm10IBAAAAABAAEAQB8AAEAfAAABAAgAZGF0YQoGAACBhYqFbF1fdJivrJBhNjVgodDbq2EcBj+a2/LDciUFLIHO8tiJNwgZaLvt559NEAxQp+PwtmMcBjiR1/LMeSwFJHfH8N2QQAoUXrTp66hVFApGn+DyvmwhBTGH0fPTgjMGHm7A7+OZURE'
  audio.play().then(() => {
    console.log('HTML Audio element test successful')
  }).catch(e => {
    console.error('HTML Audio playback failed:', e)
  })
}

// Test MediaSource with minimal WebM
export function testMediaSource() {
  console.log('Testing MediaSource...')
  
  const audio = new Audio()
  const mediaSource = new MediaSource()
  audio.src = URL.createObjectURL(mediaSource)
  
  mediaSource.addEventListener('sourceopen', () => {
    console.log('MediaSource opened')
    try {
      const sourceBuffer = mediaSource.addSourceBuffer('audio/webm; codecs="opus"')
      console.log('SourceBuffer created successfully')
      
      // Try with a minimal WebM header
      const webmHeader = new Uint8Array([
        0x1a, 0x45, 0xdf, 0xa3, // EBML
        0x9f, 0x42, 0x86, 0x81, 0x01, 0x42, 0xf7, 0x81, 0x01, 0x42, 0xf2, 0x81, 0x04,
        0x42, 0xf3, 0x81, 0x08, 0x42, 0x82, 0x84, 0x77, 0x65, 0x62, 0x6d, 0x42, 0x87,
        0x81, 0x02, 0x42, 0x85, 0x81, 0x02
      ])
      
      sourceBuffer.appendBuffer(webmHeader)
      console.log('Appended WebM header')
      
    } catch (e) {
      console.error('SourceBuffer error:', e)
    }
  })
  
  document.body.appendChild(audio)
}