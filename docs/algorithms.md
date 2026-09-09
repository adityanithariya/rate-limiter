# Algorithms:

1. Fixed Window Counter:
   In Fixed window counter, time is divided into fixed window, and rate limiting will be done over that window
   Config: 5000 reqs/min
   17:00 - 17:01: 500 requests can be allowed
   17:01 - 17:02: 500 requests can be allowed
   If someone sends burst requests at 17:00:50 - 17:00:59, completely utilizing 500 req/min
   It will again be allowed as time changes to 17:01:00, and if burst continues, and again quota is exceeded
   Inefficiency: This algorithm allowed 1000 reqs within 20sec or so, due to limited window and restricted smoothness
2. Sliding Window Log
   In Sliding window log, resource acquisition is logged for all requests under specified window, which slides over time
   Config: 5000 reqs/hour
   ```
     A(17:00) ------- B(17:01) -------- C(17:05) ------------- D(17:09) - - - - - - - X(17:59) ------------ Y(18:00) ------------ Z(18:01)
     ^
     Discarded as Z request comes in, as A.time - current.time > config.time
   ```
   This ensures smoothness over time and allows request gradually after quota exceed
   But uses significant space to log and store all the requests
3. Sliding Window Counter
   Sliding Window Counter combines Fixed Window Size & Sliding Window Log by their capabilities
   Reusing the concept of just storing the count of requests accumulated for a particular time interval
   Smoothing the count into the way slider log keeps track of certain window
   Config: 5000 reqs/min
   ```
   17:00 --------------------17:00:55----- 17:01 ----17:01:10------------------------- 17:02
                               ^                        ^
                             5k reqs                  5k reqs
   ```
   This broke fixed window size smoothness, but its solved by smoothing the last window request count to allow limited count in current window
   Estimated request in config window: current window count + last window count * (1 - progress of current window)
   Progress of current window: (current time - current window start time) / window size
   For our example:
   ```
   progress = (17:01:10 - 17:01:00) / 60 (1min) = 10/60 = 0.167
   requests = 800 + 5k (1 - 0.167) = 4966 (under allowed reqs, req allowed)
   ```
4. Token Bucket
   Token Bucket is slightly different than previous ones, instead of monitoring how many requests comes in
   It generates tokens in a bucket which requests consume, and once all tokens are consumed, rate limit is hit.
   Bucket is refilled with a fixed rate, by which it achieves the smoothest way of distributing requests over the configured window
   But it allows burst requests to come in, which might cause thundering herd or similar crashes
   Bucket refill is done only at the time of rate limit check, as we can calculate how many tokens accumulated in that time difference
   Config: 5000 reqs/min
   Token consumption is straightforward, refilling is done as follows:
   Total capacity B, Refill rate r, time elapsed between current and last request t
   new tokens = min(B, old tokens + t * r)
5. Leaky Bucket
   Leaky Bucket implements burst handling with smoothness, by implementing a queue which processes request at a steady rate
   If requests exceed max capacity/bucket size, requests are dropped.
