
export interface Entry {
  title: string;
  username: string;
  password: string;
}

export const mockEntries: Entry[] = [
  { title: 'GitHub', username: 'yoshi-dev', password: 'muh-1234' },
  { title: 'Email', username: 'yoshi.kuh@example.com', password: 'grass-is-green' },
  { title: 'Bank', username: 'DE-4471-2093', password: 'milk&cookies!' },
  { title: 'Netflix', username: 'moovie.night@example.com', password: 'Moo!Flix#2026' },
  { title: 'Spotify', username: 'cowbell_beats', password: 'cowbell-4ever' },
  { title: 'Amazon', username: 'hay.shopper@example.de', password: 'Hay$Bale_77' },
  { title: 'Discord', username: 'UdderChaos#0420', password: 'udder.chaos.9' },
  { title: 'Steam', username: 'pasture_gamer', password: 'pasture-gamer-42' },
  { title: 'Reddit', username: 'u/holy_cow_99', password: 'r/cows<3moo' },
  { title: 'Google', username: 'grazing.field@gmail.example', password: 'Gr4zing!Field' },
  { title: 'University Portal', username: 's1234567@uni.example.de', password: 'semester_Kuh_WS26' },
  {
    title: 'Very Long Website Name For Testing',
    username: 'a.very.long.username.for.layout.testing@example.com',
    password: 'xK9#mP2$vL7@qR4!nB8&wT5^zF3*hJ6',
  },
];
