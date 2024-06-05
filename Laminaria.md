# Synthé midi, compagnon ORCA

3 composantes :
 - recevoir du midi et l'envoyer à l'interface et au dsp (channel)
 - recevoir des input interfaces et les envoyer au dsp (channel)
 - déclencher une voix midi sur le note On avec les paramètres de l'interface et du midi (on verra plus tard pour l'évolution et la polyphonie)

 - [x] Aller chercher le morceau de code pour les channels et l'asynchronie
 - [x] réussir à ouvrir un sine avec un note On, la fermer sur un note Off.
 - [ ] Faire l'interface à part
 - [x] aller chercher le bout de code pour l'interface CLI
 - [ ] Polyphonie
 - [ ] Definir la taille du terminal quand j'aurais fini l'interface ?
 - [ ] specific type of parameter for mode handling
 - [ ] Enveloppe
 - [x] replace ui channel struct by an enum

## Interface :
parameter1 - a - ||||||||||---------
**parameter2 - 1 - |-------------** <-- selected
parameter3 - z - |||||||||||||||||||||

arrow up down -> select next and previous
arrow left right -> increase / decrease parameter
number or number -> set value

faire une classe paramètre avec un label, en faire un array, le faire apparaitre un par un, permet de les réarranger rapidement dans le display

pour mettre en forme use `use colored::Colorize;`, puis `println!("{}", string.bold())`

[asyncronicity dans rust](https://rust-lang.github.io/async-book/05_streams/01_chapter.html)
[reference synth](https://github.com/chris-zen/kiro-synth)

`Stream` est un *Trait* 

`async` est un type de fonction non bloquante, `Future` est un trait permettant de les executer

Utiliser des `async` et des await -> redéclare la fonction à chaque fois ? Pas terrible non ? 
mettre la boucle dans la fonction async, pas sûr que ça marche bien

Le `midir` crée tout seul son thread, mais besoin de l'attribuer à une variable qui reste existante, faudrait le passer à la fonction main

## midi message
⚠️ you got to be in `$cc:0` in Orca
NoteOn = 144
NoteOff = 128
CC = 176


## Composition vs inheritance

Composition means *HAS* A
Inheritance means *IS* A

Example: Car has a Engine and Car is a Automobile