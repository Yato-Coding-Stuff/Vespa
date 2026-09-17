# Profiles

Silksong, through its use of bepinex doorstop loading, allows you to easily seperate out profiles without ever touching the game files.
Hollow Knight, through its custom api, does not.

To still allow Hollow Knight to use profiles, we need to make a general solution that allows for easy profile management no matter what game is using it.
For this, i suggest the following:

# Transaction

The profile system needs to be able to take files out of the currently selected profile, 
and on run, copy these files into the game folder( see also [#Prime](#Prime-Command)).
Once either the run is finished, or some kind of unexpected error occurs, these files need to be removed and original state needs to be restored.

To ensure this, we need to, on run( or [#Prime](#Prime-Command]), build some form of "Transaction" Structure that contains all the necessary information to restore the game to its original state.
This structure will need to be stored to disc while the game is running, to allow restoration upon the next run in case of an error causing premature termination.
This structure should be removed on a successful run.

Transaction:
```json
{
    "game": "Hollow Knight",
    "profile": "Default",
    "files": [
        {
            "profile_path": "/path/to/file",
            "game_path": "/path/to/file",
            "backup_path": "/path/to/file",
            "existed": true
        },
        {
            "profile_path": "/path/to/file",
            "game_path": "/path/to/file",
            "backup_path": null,
            "existed": false
        }
    ]
}
```

This structure stores the following information:

* Game name
* Profile name
* A list of files that need to be restored
* * The path of the file in the profile that gets copied to the game directory
* * The path of the file in the game directory
* * The path of the file in the backup directory, will be null if the file did not exist in in the game_path beforehand
* * Whether the file existed in the game_path beforehand, true or false

# Prime-Command

Prime allows Vespa-managed profiles to be indepdented of the modinstaller itself. 
Once a profile has been primed, the game can be launched through any mechanism, such as through steam directly, without requiring you to use Vespa at all.

To do this, we will use the same Transaction-Mechanism as before, we will simply skip the cleanup of the transaction and persist it to disk.

A primed profile is a snapshot of the profile's state the the time 'prime' was called. 
Any operation that changes that state will thus invalidate the currently primed state.

To ensure correctness, any command that changes profile state will thus first restore the primed state, execute the command, and then prime the new state.
This greedy approach will lead to unnecessary file i/o, and might be changed out for a selective approach in the future
