\babel@toc {english}{}\relax 
\contentsline {lstlisting}{\numberline {4.1}{\ignorespaces State Machine struct.}}{36}{lstlisting.101}%
\contentsline {lstlisting}{\numberline {5.1}{\ignorespaces Structs added}}{41}{lstlisting.138}%
\contentsline {lstlisting}{\numberline {5.2}{\ignorespaces How to implement an impl for the struct}}{42}{lstlisting.198}%
\contentsline {lstlisting}{\numberline {5.3}{\ignorespaces How to calculate the value of the SV's}}{43}{lstlisting.213}%
\contentsline {lstlisting}{\numberline {5.4}{\ignorespaces How to calculate the value of the SV's}}{43}{lstlisting.225}%
\contentsline {lstlisting}{\numberline {5.5}{\ignorespaces EthernetFrame struct.}}{44}{lstlisting.237}%
\contentsline {lstlisting}{\numberline {5.6}{\ignorespaces The main logic and structure of the state machine are implemented here, showcasing the tick time, state evolution, log messages, and transitions.}}{46}{lstlisting.254}%
\contentsline {lstlisting}{\numberline {5.7}{\ignorespaces This function handles invalid samples by comparing their value to N\_SAMPLES. If less, it increments a counter and transitions to CompleteSample, if greater or equal, it moves to ToggleMU. This must happen before reaching CompleteCycle, where variables are reset.}}{48}{lstlisting.348}%
\contentsline {lstlisting}{\numberline {5.8}{\ignorespaces This function handles questionable samples by simply waiting for the next tick to transition to the CompleteSample state.}}{48}{lstlisting.370}%
\contentsline {lstlisting}{\numberline {5.9}{\ignorespaces This function processes valid samples by summing and dividing buffer values. The error is calculated as the ratio of buffer 1 to the total of both buffers and vice-versa. If the error percentage is less than 25\%, it transitions to CompleteCycle; otherwise, it goes to ToggleMU.}}{49}{lstlisting.380}%
\contentsline {lstlisting}{\numberline {6.1}{\ignorespaces First scenario showing the steps through the state machine between the state Get Sample -> Valid, Valid -> Complete Sample -> Get Sample.}}{57}{lstlisting.475}%
\contentsline {lstlisting}{\numberline {6.2}{\ignorespaces Second scenario showing the steps through the state machine between the state Get Sample -> Valid, Valid -> Check the Error Percentage -> Toogle MU -> Complete Cycle -> Get Sample.}}{57}{lstlisting.501}%
\providecommand \tocbasic@end@toc@file {}\tocbasic@end@toc@file 
