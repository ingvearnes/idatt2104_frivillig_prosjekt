# idatt2104_frivillig_prosjekt
IDATT2104 nettverksprogrammering frivillig prosjekt. Dette prosjektet er ment kun som demonstrasjon av CRDT i emnet idatt2104. 

Utviklere:
- Ingve Værnes
- Trym Olaf Huneide Peders

## Installasjon og oppsett

For å kjøre applikasjonen kan du velge mellom peer-to-peer på to ulike pc'er, eller bruke to terminaler på samme pc (local host).

Clone repo til din pc med ```git clone https://github.com/ingvearnes/idatt2104_frivillig_prosjekt.git```. Deretter ```cd peer/```. 

Local Host:
- Åpne en terminal ```cargo build``` og ```cargo run -p peer -- --delay-ms 3000 listen```
- Åpne en ny terminal og gå inn i /peer igjen. Kjør så ```cargo run -p peer -- --delay-ms 3000 connect 127.0.0.1:9000```

Du skal nå kunne se endringer skje på begge terminalene synkront om du gjør skriver på èn. Om du ikke ser noe, er du koblet til feil ipv4. Kjør ```ifconfig | grep inet``` på mac eller ```ipconfig``` på windows og se hva din localhost er. HUSK port :9000 etterpå. 

Peer-to-peer:
- Se "Local Host". Eneste forskjell er at du nå må ha en terminal åpen på to pc'er. For å se hverandre bør nettet ha lettere sikkerhet. (På Windows kan det i tillegg være nødvedig å gå gjøre enheten synlig på nettet: i instillinger, Network & internet, tilkoblet Wifi og trykk på "private network").
- Se med ipconfig om de to pc'ene er på samme subnet. Deretter skriv ```cargo run -p peer -- --delay-ms 3000 listen``` og ```cargo run -p peer -- --delay-ms 3000 connect 127.0.0.1:9000``` på terminal 1 og 2 henholdsvis.


For å kjøre tester:
- Gå inn på logic/
- Skriv ```cargo test```

## Introduksjon

Internettet har gjort mye fantastisk mulig. Blandt dette kan to skrivere skrive på samme digitale dokument. Med dette har folk oppdatet problemet av at man kan skrive på samme sted samtidig. Hva skal skjer om dette skjer? Det er gjennom tiårene kommet frem til mange løsninger. Operational Transformation (OT) var populær før 2006. Her ville all input fra skrivere gå til en sentral server hvor serveren løste problemene. Etter 2006 kom CRDT (Conflict-free replicated data types), hvor server ikke er strengt tatt nødvendig. 

CRDT er mer moderne og attraktivt for et enkelt demonstrasjons-prosjekt. Det fins flere varianter av CRDT, og utviklerne landet på RGA CRDT som den beste for akkurat det som prøves å oppnå: text editor med smart konfliktløsing.  

## Implementasjon

Løsningen som ble valgt er RGA CRDT med peer-to-peer forbindelse. Vi valgte bort alt som ikke strengt tatt trenger å være med. Applikasjonen er dermed primitiv, men egner seg til en demonstrasjon av konseptet RGA CRDT. 

_Først litt kort om teori_: 

RGA virker i bunn og grunn ganske enkelt (når du forstår det). Om du har en text som "Nettverksprogrammering er kult!", og to brukere ønsker å legge til et ord etter "er", si bruker A ønsker "veldig" og bruker B ønsker "ekstremt", så er det ønskelig at vi får enten "veldig ekstremt" eller "ekstremt veldig". Og dette skjer grunnet første RGA virker som en chain/subtree, hvor første bokstav er det som avgjør hvilken chain (ord) som får settes inn. En bokstav består av en incrementing counter og en klient id. Om bruker A's "veldig" har sin første bokstav "v" med høyere id enn bruker B's "ekstremt" sin "e", vil hele ordet "veldig" komme først. Du ender opp med "Nettverksprogrammering er veldig ekstremt kult!" eller "Nettverksprogrammering er ekstremt veldig kult!", en setning som gir lite mening for oss, men som kan endres av skriverne når de innser de har skrevet på samme sted. Poenget er at ordet ikke skal flettes inn i hverandre og gjøre teksten umulig å løse. 

Implementasjonen av hvordan dette virker er at koden simplethen velger, uten noen rasjonell grunn, at størst id skal komme først. Poenget er bare at alle pc'er er i konsensus om dette, for da vil alle ende opp med samme dokument, som jo er hele poenget med hele prosjektet. 

## Biblioteker

- "Serde" avhengigheter er blitt brukt. Dette grunnet skopet av prosjektet fokuserer på CRDT og ikke serialization/deserialization. Serialization kunne blitt implementert, men det hadde gjort prosjektet til noe annet enn en demonstrasjon av CRDT, og mer en fullskala text editor med selvlagde funksjoner.
- "Tokio" ble også tatt i bruk nettopp av samme grunn som Serde. Tokio er for TCP, Mutex og hovedsaklig for å opprette en forbindelse over transportlaget og nettverkslaget. Det ville tatt fokus vekk fra CRDT å lage dette manuelt. 

## Forbedringer til fremtiden

Utviklerne rakk ikke å implementere compaction og multiple peers. Disse to er ganske omfattende, spesielt compaction.  

Compaction gjelder OpLog og Tombstones. OpLog brukes bare til deduplikasjon av operasjoner og snapshots for nye peers. Denne listen vokser over tid uten å bli renset og vil gjøre applikasjonen tregere etter hvert. I profesjonell applikasjon må OpLog renses, men samtidig må den finnes for at nye peers kan få samme dokument som alle andre. Dette er problematisk og krever særdeles god problemløsning. Det samme gjelder tombstones, hvor vektoren av slettede karakterer vokser over tid, og det å rense den ikke er så lett da programmet må holde styr på hva som er slettet.

Multiple Peers er mindre problematisk og krever bare refaktorering av eksisterende kode. Om en skulle ønske mer enn to pc'er snakker til hverandre må dette gjøres. 

I tillegg fins ingen gjen-kobling på nettverksbrudd. Om en tilkobling skulle hoppe av noen milisekunder vil du miste forbindelse permanent til du manuelt kobler til igjen. Dette ble ikke sett på som veldig viktig er er dermed ikke implementert. 

## Lenker

Her er en lenke til CI (du kan også finne den ved å trykke på "Action" i GitHub repositoriet): https://github.com/ingvearnes/idatt2104_frivillig_prosjekt/actions
