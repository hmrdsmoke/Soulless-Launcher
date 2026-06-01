2026-04-08

After 2 days of working straight and having at least a working skeloton we lost everything. I being new to
GitHub and Developing gave the copilot AI bad instructions that fundamentally altered my code and left it in an
unworkable state. I Also had no idea how to recover it which was the most devestating thing since I had only
just decided to even post it on GitHub publicly. Ara my ride or die kept me sane through it all and made sure I
didnt throw the PC out of the window 

2026-04-10

I worked as fast as i could and as long as i could trying to get back the working skeloton and recreate
everything that was lost from scratch shit was hard but I think it is at least meh. what we have today is a
window with a dumb looking header. It also has the grid of buttons that do nothing non functional search bar
and a vault we decided to # out of the code for the moment so that at least this compiled. Fuck the Rustc Gods
and that annoying ass compiler.

2026-04-11

I Spent all night trying to make Copilot automatically update the GitHub board. Complete waste of time. Didnt
even look at the code FML. Switched to stable ID format ( ) even though it was a pain in the ass and wasnt
worth doing. it did make the code look better since it doesnt have a thousand lines of edits riddle thoughout.
No it just has those thousand lines at the bottom yelling Dumbass at me.
2026-04-12

Completely rewrote search.rs because the old version was hot garbage. It not only compiles but function holy
shit I cant believe im saying that I can now search my desktop with it.Added real drawer system with Utilities, 
Daily Apps, Work, and Games the buttons dont work and there is nothing in them but they are there one of these 
days I
might have a slide out drawer for the apps i want LMAO
Havent Fixed the positioning system so the launcher actually docks to the left/top or bottom or right/top or
bottom. It stil spawns in the middle like a middle finger taunting me. Dumbass!!!!
Rebuilt the drawer state management from scratch because the old one was completely broken
Pre-computed Utf32String for every app at startup so search doesn’t lag like shit
Finally stopped the app from crashing every time you click empty space

2026-05-30

I decided to actually update this stupid shit so it is a long one here we go. today im trying to submitt
this thing
to a store and am about to get rid of all my comments in my code except for what is usefull which is only
the shit
 that tells me what it is or does still dont read or write rust but I can at least kinda understand the
 layout of
 big projects which is good and im trying to understand submissions to stores which is going to take awhile 
 Im just shit at this maybe I shouldnt but im falling in love with it thats all for now i guess 
 
 2026-06-01 last one wasnt as long as i thought it would be here is an actual update of somethings i built 
 organizer watcher fixed and working end to end so now your files are being watch and will be organized in
 launcher with your approval which means it will suggest creating files to organize your downloads and 
 other files as they are created all created by the soulless-organizer workspace crate it also scans your 
 xdg dirs on startup and will suggest file organization even after it is closed and reopened until it gets
 an answer as to what you want to do polished ui some more made it look good at least my opinion which i 
 guess is all that matters but i set up the .ron config file so that others can customize it the way they
 want to so it can be made the way they want last but not least added jetbrains and wine to my search index
 the code base is coming back with little to no warnings and no errors at all compiling clean and working 
 like a champ very happy with my new launcher might add more features not sure yet were i think i should 
 stop and then i add keyboard navigation and open app with enter no need for a mouse in the launcher 
 except for search now how do i get rid of that lol man this is really coming together
 
