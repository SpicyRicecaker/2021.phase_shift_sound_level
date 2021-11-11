# Program

The goal of this program is to play two identical sine waves through two speakers at different phase shifts. We should be able to define the degree of phase shift as well as frequency of the wave

## How do we generate phase shift?

There's 3 main ways we could generate phase shift: 
1. Create an audio file on audacity, and just play it directly
    - This is probably the easiest and least techically difficulkt, but the most lame way, as it is hard to automate
2. Play 2 different audio files / sine waves through 2 different sound streams, delayed at certain intervals
    - This method **doesn't work**. To achieve a desired phase shift between a single wavelength for a 440 Hz wave (equivalent of A note), `1ms` accuracy is needed. The playback library `rodio` doesn't support 2 channel playback, and having multiple sink playback already causes delays in startup larger than `1ms`. May be possible with a lower level library, but there would still be a large degree of error in startup. 
3. Use 1 audio stream with different data through the left and right channel
   -  This is the method we're going with. With `wasapi-rs`, we can generate our wavelength and sample bytes manually, as well as write to the L and R channels separately. The only hard question left to solve is how to cache an iterator's calculations, which is fairly simple.
