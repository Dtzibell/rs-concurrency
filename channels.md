# Ideas for channels
Thread for file parsing:
- Reading thread:
    - Create a BufReader from a file and a supporting Vector::with_capacity(16)
    - Cycle 16 lines, push each to the vector.
- Parsing threads:
    - Send the vector to a processing thread via a mpsc::channel.
- 
