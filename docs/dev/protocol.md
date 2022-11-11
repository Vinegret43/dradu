
# Protocol versioning

Protocol version is in form of `MAJOR.MINOR`. MAJOR is incremented when changes
are backwards-incompatible, MINOR is incremented when they aren't

**Note: while this is in alpha, version always stays at 0.1**


# General message structure

First line is the heading, in form of `dradu/<PROTOCOL VERSION> <MESSAGE TYPE>`

Properties are written in `key:value` form (There's no space in between)

Header is separated from message body with one empty line

```
dradu/<VERSION> <MESSAGE TYPE>
<KEY>:<VALUE>

<BODY>
```


# Message types

Can be in caps or not, matching them shouldn't be case-sensitive

### Client message types
 - **JOIN**  
  Sent as the very first mesage when connecting to an already existing room  
 *Properties:*
  ```
  contentType:json
  ```
 *Body:*
  ```
  {
   "roomId": "<ID of the room you wanna connect to>"
  }
  ```

 - **INIT**  
  Create a new empty room. Player who creates it will be given full permissions
  to everything (e.g. he will be the GM)  
  *Properties:* none  
  *Body:* none

 - **QUIT**
  Signals that you are quitting the game. After sending this server won't
  process you anymore, so you should close the socket connection  
  *Properties:* none  
  *Body*: none

 - **MAP**
  Update map state (Move, add, delete, resize an object, background image, etc.)  
  *Properties:*  
   This action requires permissions, so you *should* include your ID and cookie
   ```
   contentType:json
   userId:<Your user ID>
   userCookie:<Your user cookie>
   ```
  *Body:*
   ```json
   {
     // Example of adding a new object
     "itemIdThatDoesntExistYet": {
       "type": "decal"/"token"/"wall"/"effect",
       "path": "path/to/image.png",  // Also see FILE message type
       "scale": 1.0,
       "pos": [x, y],
     },

     // Example of moving an object. Same thing for rescaling (You can include
     both "pos" and "scale" simultaneously)
     "existingItemId": {
      "pos": [new_x, new_y],
     },

     // This is how you delete an object - just provide an empty dictionary
     "existingItemId": {},

     // You can update/add/delete as many objects as you wish
     "anotherItemId": {
       ...
     },
     ...
   }
   ```

 - **MSG**  
  Send a chat message. There may also be chat commands (Usually starting with
  a slash), but this depends on the server  
  *Properties:*  
   ```
   contentType:text
   userId:<Your user ID>
   userCookie:<Your user cookie>
   ```  
   *Body:*
    Normal utf8-encoded text

 - **FILE**  
  Send or request a file. If you are a normal player (Not the master), you can
  request a file which is referenced in "path" of a map object (1), server will
  then send the same request to the master, master will answer this request (2)
  and send another FILE to the server with contents of the request file in body,
  then the server will redirect that file to you, also using FILE type. i.e:
   Client --> Server --> Master --> Server --> Client  
  (1) When requesting a file:  
   *Properties:*  
   ```
   path:<Path to the file>
   ```  
   *Body:*
    none  
  (2) When answering a file request:  
   *Properties:*  
   You need to specify `path` because there might be *many* pending file requests
   ```
   path:<Path to the file which was requested>
   contentType:<image/audio(WIP)>
   ```  
   *Body:*
    binary bytes of the requested object

 - **PERM**  
  WIP
   

### Server message types

DRADU doesn't really implement client-server architecture, even though it claims
to do so. This is because communication model isn't just plain request-response:
when there is an update to the map, a new chat message, etc., server will send
you a message, even though you didn't make a request, so you have to *always*
listen for incoming messages

 - **OK**  
  Can be sent in response to the following client requests:
   - - Join  
    *Properties:*  
    ```
    contentType:json
    ```  
    *Body:*  
     ```json
     {
      "userId": "User ID assigned to you by the server",
      "userCookie": "Your user cookie",
      "color": "Color of your nickname, assigned by the server",
      "nickname": "Your nickname, assigned by the server",
     }
     ```
   - - Init
    *Properties:*
    ```
    contentType:json
    ```
    *Body:*
     ```json
     {
      "userId": "User ID assigned to you by the server",
      "userCookie": "Your user cookie",
      "color": "Color of your nickname, assigned by the server",
      "nickname": "Your nickname, assigned by the server",
      "roomId": "Id of the room you just created",
     }
     ```

 - **MAP**  
  Update to the map state.
  *Properties:*
   ```
    contentType:json
   ```
  *Body:* Completely the same as in client's (See *Client message types > MAP*)

 - **FILE**  
  If you are the master, this is a file request (1). If you aren't - this is
  a response to your file request (2)
  (1)
   *Properties:*
   ```
   path:<Path to the file>
   ```
   *Body:* none
  (2)
   *Properties:*
   ```
   path:<Path to the file>
   contentType:<image/audio(WIP)>
   ```
   *Body:* 
    binary bytes of the requested object

 - **MSG**  
  *Properties*:  
   ```
   userId:<Sender ID>
   contentType:text
   ```  
   *Body:* Normal utf8-encoded text

 - **PLAYER**  
  Update to the list of players or player properties (Color, nickname).
  *Properties:*
   ```
   contentType:json
   ```
  *Body:*
   ```json
   {
    // To add a new player
    "playerId": {
     "nickname": "player_nickname",
     "color": [r, g, b],
    },

    // To update players properties. Include "nickname", "color", or both
    "existingPlayerId": {
     "nickanme": "new_nickname",
     "color": [r, g, b],
    },

    // Deleting a player from the list (e.g. in case of disconnect)
    "existingPlayerId": {},
    // You can update as many player as you wish in one message
    "existingPlayerId": {
     ...
    },
    ...
   }
   ```

 - **QUIT**  
  Sign that the server just disconnected you from the game. Also, when *you*
  send QUIT, the server will answer it with this request, though you don't have
  to process it, you can disconnect right away  
  *Properties:* none
  *Body:* none. TODO: Should provide some sort of reason

 - **SYNCED**  
  WIP

