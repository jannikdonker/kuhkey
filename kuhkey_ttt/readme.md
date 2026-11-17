# ttt - track, trust, two-factor

checks where you log in from on `unlock`

- **track**: asks ipinfo.io for your country, saves a counter per country + the last 100 logins in `locations.json`
- **trust**: under 10 logins we just learn, known country => let in
- **two-factor**: new country => 2fa (todo, blocked for now)

offline or ipinfo down => check gets skipped, psw manager has to work without internet

<img width="1920" height="1080" alt="TTT – Logo@2x" src="https://github.com/user-attachments/assets/1363e1e0-04c5-483f-a1bd-638c7f26322d" />
