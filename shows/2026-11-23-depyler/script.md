# 2026-11-23-1-demo — depyler: ship the Python gate as one Rust binary

## OPEN

Yesterday's show was about deploys. Here's the smallest piece of one. A canary gate. Ten requests hit the new version. If too many fail, roll back. If not, promote.

Every team writes this in Python. Today we ship it as one Rust binary, with depyler.

## SHOW THE GATE

$ cat canary.py

Twenty lines. Count the 500s. Compare them to a budget. Print promote or rollback. Typed Python. That matters in a second.

$ python3 canary.py

Two errors out of ten. The budget is one percent. Rollback. Good. The gate works.

## LINT

$ depyler lint canary.py

"canary.py is Depyler Python compliant." Depyler wants typed, plain Python. No magic. If your code passes this, it can become Rust.

## COMPILE

$ depyler compile canary.py

One command. Python in, a native binary out. Under the hood it writes Rust and runs cargo.

$ ls -la canary

About 440 kilobytes. No Python. No virtual environment. No pip install on the build box.

$ ./canary

Same answer. Two of ten. Rollback.

## WHY IT MATTERS

Now time them. Fifty runs each.

$ time (for i in $(seq 50); do python3 canary.py >/dev/null; done)

$ time (for i in $(seq 50); do ./canary >/dev/null; done)

Read the two numbers. On my test box Python took about three quarters of a second. The binary took about seven hundredths. Roughly ten times faster, and almost all of that is Python starting up.

That's the point for a deploy pipeline. A gate runs on every change. Thousands of times a day. It should start instantly and it should have nothing to install. The fewer moving parts at the door to production, the fewer ways the door jams.

## LOOK AT THE RUST

$ depyler transpile canary.py

$ grep -n "fn gate" -A 4 canary.rs

Here's the gate in Rust. Same logic. Typed integers, a plain bool out. The file is about 3,400 lines, because depyler ships a small runtime for Python behavior inside it. You don't edit that file. You edit the Python and recompile.

## HONEST NOTE

It's not magic. Depyler handles a subset of Python. Typed functions, lists, loops, f-strings. If lint says no, fix the Python first. That's the deal.

## CALL TO ACTION

Cargo install depyler. The repo is github dot com slash paiml slash depyler. If you want the full course on moving Python to Rust, it's on Coursera and edX. Links are below. The newsletter is ds500.paiml.com. I'm Noah Gift. Go compile a gate today.
