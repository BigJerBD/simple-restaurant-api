Requirements
==========
* The client (the restaurant staff “devices” making the requests) MUST be able to: add one or more items with a
  table number, remove an item for a table, and query the items still remaining for a table.
* The application MUST, upon creation request, store the item, the table number, and how long the item will take to cook.
* The application MUST, upon deletion request, remove a specified item for a specified table number.
* The application MUST, upon query request, show all items for a specified table number.
* The application MUST, upon query request, show a specified item for a specified table number.
* The application MUST accept at least 10 simultaneous incoming add/remove/query requests.
* The client MAY limit the number of specific tables in its requests to a finite set (at least 100).
* The application MAY assign a length of time for the item to prepare as a random time between 5-15 minutes.
* The application MAY keep the length of time for the item to prepare static (in other words, the time does not have
  to be counted down in real time, only upon item creation and then removed with the item upon item deletion).

## Allowed Assumptions

You may have your application assume the following to simplify the solution, if desired:

* The time to prepare does not have to be kept up-to-date.  It can also just be generated as some random amount
  of time between 5 and 15 minutes and kept static from then on.
* The table and items can be identified in any chosen manner, but it has to be consistent. So if a request comes in
  for table "4", for example, any other requests for table "4" must refer to the same table.
* “Clients” can be simulated as simple threads in a main() function calling the main server application with a
  variety of requests.  There should be more than one, preferably around 5-10 running at any one time.
* The API is up to the developer.  HTTP REST is acceptable, but direct API calls are also acceptable if they mimic an
  HTTP REST-like API (e.g. api_call1(string id, string resource), etc.).

