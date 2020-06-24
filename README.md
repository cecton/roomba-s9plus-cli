Roomba S9+ CLI
==============

Installation
------------

```
cargo install roomba-s9plus-cli
```

Usage
-----

In order for it to work you will need to find out a few information:

1. The IP address of the device
2. The user and password
3. Optional: if you want to be able to clean a room or a set of rooms, you will
   need the `pmap_id` and `user_pmapv_id`.

### Find the IP address

You can use the command:

```
roomba-s9plus-cli find-ip
```

This command will run indefinitely and show all the Roomba thingies on your
network.

### Find the user and password

### Find the `pmap_id` and `user_pmapv_id`
