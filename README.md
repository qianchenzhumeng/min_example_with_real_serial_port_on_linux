An [min-rs](https://github.com/qianchenzhumeng/min-rs) example with real serial port on linux.

## usage

```bash
RUST_LOG=trace cargo run
```

## Ardurion UNO

source: [https://github.com/min-protocol/min/tree/master/target/sketch_example1](https://github.com/min-protocol/min/tree/master/target/sketch_example1) 

```c++
#include "min.h"
#include "min.c"

struct min_context min_ctx;

// This is used to keep track of when the next example message will be sent
uint32_t last_sent;

////////////////////////////////// CALLBACKS ///////////////////////////////////

void min_tx_start(uint8_t port){

}

void min_tx_finished(uint8_t port) {
}

// Tell MIN how much space there is to write to the serial port. This is used
// inside MIN to decide whether to bother sending a frame or not.
uint16_t min_tx_space(uint8_t port)
{
  // Ignore 'port' because we have just one context. But in a bigger application
  // with multiple ports we could make an array indexed by port to select the serial
  // port we need to use.
  uint16_t n = Serial.availableForWrite();

  return n;
}

// Send a character on the designated port.
void min_tx_byte(uint8_t port, uint8_t byte)
{
  // Ignore 'port' because we have just one context.
  Serial.write(&byte, 1U);  
}

// Tell MIN the current time in milliseconds.
uint32_t min_time_ms(void)
{
  return millis();
}

// Handle the reception of a MIN frame. This is the main interface to MIN for receiving
// frames. It's called whenever a valid frame has been received (for transport layer frames
// duplicates will have been eliminated).
void min_application_handler(uint8_t min_id, uint8_t const *min_payload, uint8_t len_payload, uint8_t port)
{
  // In this simple example application we just echo the frame back when we get one, with the MIN ID
  // one more than the incoming frame.
  min_id++;
  // The frame echoed back doesn't go through the transport protocol: it's send back directly
  // as a datagram (and could be lost if there were noise on the serial line).
  min_send_frame(&min_ctx, min_id, min_payload, len_payload);
}

void setup() {
  // put your setup code here, to run once:
  Serial.begin(115200);
  while(!Serial) {
    ; // Wait for serial port
  }

  // Initialize the single context. Since we are going to ignore the port value we could
  // use any value. But in a bigger program we would probably use it as an index.
  min_init_context(&min_ctx, 0);

  last_sent = millis();
}

void loop() {
  char buf[32];
  char min_payload[] = {0, 1, 2};
  size_t buf_len;

  // Read some bytes from the USB serial port..
  if(Serial.available() > 0) {
    buf_len = Serial.readBytes(buf, 32U);
  }
  else {
    buf_len = 0;
  }
  // .. and push them into MIN. It doesn't matter if the bytes are read in one by
  // one or in a chunk (other than for efficiency) so this can match the way in which
  // serial handling is done (e.g. in some systems the serial port hardware register could
  // be polled and a byte pushed into MIN as it arrives).
  min_poll(&min_ctx, (uint8_t *)buf, (uint8_t)buf_len);

  // Every 1s send a MIN frame using the reliable transport stream.
  uint32_t now = millis();
  // Use modulo arithmetic so that it will continue to work when the time value wraps
  if (now - last_sent > 1000U) {
    // Send a MIN frame with ID 0x33 (51 in decimal) and with a 4 byte payload of the 
    // the current time in milliseconds. The payload will be in this machine's
    // endianness - i.e. little endian - and so the host code will need to flip the bytes
    // around to decode it. It's a good idea to stick to MIN network ordering (i.e. big
    // endian) for payload words but this would make this example program more complex.
    if(!min_queue_frame(&min_ctx, 0x33U, (uint8_t *)&now, 4U)) {
      // The queue has overflowed for some reason
      //Serial.print("Can't queue at time ");
      //Serial.println(millis());
    }
    min_send_frame(&min_ctx, 1, min_payload, 3);
    last_sent = now;
  }
}
```

## troubleshooting

[1] [rust serial programming](https://qianchenzhumeng.github.io/posts/rust_serial_programming/)