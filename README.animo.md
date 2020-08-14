## Inspiration
Most time, I’m developing applications, for example https://mymem.me/ 
Evolutionarily, I build up pattern that cover my need for ~95% and allow me to speed up build process a lot. 
It’s not a secret, any (or almost any, semantical for sure) can be represented by a graph. Nodes can be named by unique ids (for example, random UUID). 
Edges also can be named by ids, but that ids from nodes ids … simple to thing of it as reference to node. 
Also, edges can be named not only by one id but number of it. The key point that order of that list of ids does not matter.
This set of data structures allow to build complex application in short time.

## What it does
Handle semantic graph mutations/evaluation.

## How I built it
Most time I spend to understand substrate infrastructure. After it was quite easy to code.

## Challenges I ran into
I did try to code UI component for it by fail so far. The work is in process. I can share code that I have right now, but it does not act as I would like it to do.

## Accomplishments that I'm proud of
I was able to run it! The hardest part was to understand how to fix something, because errors was so meaningless.

## What I learned
Substrate ;-)

## What's next for Animo
Semantic graph itself is useful as core/trusted information source, but it’s can be very slow on data querying.
To speed up it only structure of request is required.
For example, there are records “movie A“ “release date” “2020-08-10” and “movie B“ “release date” “2020-08-12”.
To get list of releases full scan required or simply build an index.
So, at time of mutation that lead to such state additional records created: 
“movie” “release date” “2020-08-10” “movie A“ and “movie” “release date” “2020-08-12” “movie B“. 
That allow to have very fast look up by the prefix “movie” “release date” and, also, it allows to have lookup in a interval, 
for example: “movie” “release date” from “2020-08-11” till “2020-08-11”.

Because mutation details available, it’s easy to keep indexes in synchronisation with core structure.