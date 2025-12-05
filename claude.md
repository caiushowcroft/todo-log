# Claude

## Overview

todo-log is a small rust, terminal application that enables people to manage projects and todos through writing logs about thier day.

The application keeps its data locally on disk in three sets of files:
todo-log/projects.yml : contains a list of projects
todo-log/people.yml : contains a list of people,
todo-log/log-<year>/<datetime>/ : a directory per log entry , where <year> is the year the log was created and <datetime> is the date and time the log was created. 

When pplication starts it presents the user with a menu:
1. Create a new log entry [c]
2. List current todos [d]
3. Show logs by project/person [s]

pressing [c] presents a new log entry screen witha timestamp of creation. The user can write a log in screen, or paste in images or text.  A user can reference a project by typing "#" then the project name, e.g. "#new-website". The application users the project slisted i the projects.yaml file to present the user with a dropdown autocomplete box as the user typoes the name of the project.

Similarly, the user can refence people from the people.yml file by typing "@" then the name, e.g. "@john". Again the application will provide autocomplete suggestions as the user types the name.  

The user can create a "todo" item by starting a new line with "[]", the rest of the line is then the todo text. The todo item inherits the project and people tags of the log entry. 

the user can attach files, images etc by dragging them into the window or typing "ctr-a" and giving the path to the file.

When the user is done, they can type "ctr-s" and the log entry is saved into a new file under projects/<year>/<datetime>/log.txt, any attachements will be copied into that directory. 

If the user selects [d] from the main menu, the application reads all log files and finds all todos (lines starting [] for an uncompleted todo and [x] for a completed todo). Each todo is tagged with all the projects tagged in the log entry and all the people in the log entry. the user is presented with a list of todos and a series of filters they can configure.
The first filter is "Show completed Y/N?", if selected then the application will show all todos regardless if they are done ("[x]") or still open ("[]").
The second filter, the user can select 1 or more projects, in which case the application will only show todos that are tagged with one or more of these projects.
Lastly the user can select 1 or more people, again the applciation will show just the todos with one or more of the selected people tagged.

The user can scroll through the list of todos withthe arrow keys and either:
* Toggle the todo "done"/"open" by pressing "x" when the todo is selected
* Show the associated log entry by pressing "l", hitting "ESC" returns them the previous view. 

When the todo is toggled to done, the application changes "[]" to "[x]" in the log file.

If the user selects [s] form the main menu they are now presented with a reverse time ordered list of logs. Again the user can filter down logs by:
* projects
* people
* dates (just show log between two dates)

By default just the first line is shown for each log entry. The user can scroll to the entry and then type "l" to show the entry. Typing "ESC" will take the user back to the list of log entries. 


## Installation

The first time the application is run, if no todo-log directory exists, it will create empty people.yml and projects.yml files with and example project and person.

Below is an example project entry:
```
  - name: new-website
    jira: https://jira.com/proejcts/WWW-123
    description: A project to create a new look on our website
    status: open
````

Below is an example people.yaml entry:
```
  - name: john
    full_name: John Smith
    email: john@example.com
    tel: 555 123 3333
    company: foo works
    
````
## Examples

[Example code or usage patterns]

## Contributing

[Contribution guidelines]

## License

[License information]
