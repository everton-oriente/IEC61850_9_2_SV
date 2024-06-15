# IEC61850_9_2_SV
Developing the minimal way to communicate in IEC61850_9_2_SV and implement in alghoritm thats choose the best analog signal.

To run the code we can not run using cargo run because we need admin permission so we can compile using cargo build, so after this it is needed to go to the folder /home/evertonoriente/Documents/IEC61850_9_2_SV/pub_iec/target/debug and run the command: sudo nice -n -19 ./pub_iec wlp0s20f3 to run the publisher.


To run the code we can not run using cargo run because we need admin permission so we can compile using cargo build, so after this it is needed to go to the folder /home/evertonoriente/Documents/IEC61850_9_2_SV/sub_iec/target/debug and run the command: sudo nice -n -19 ./sub_iec wlp0s20f3 to run the subscriber.
