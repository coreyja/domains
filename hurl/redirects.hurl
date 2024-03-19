GET http://localhost:3000
HOST: coreyja.tv
HTTP 303
[Asserts]
header "Location" == "https://coreyja.com/videos"

GET http://localhost:3000
HOST: coreyja.blog
HTTP 303
[Asserts]
header "Location" == "https://coreyja.com/posts"

GET http://localhost:3000
HOST: coreyja.club
HTTP 303
[Asserts]
header "Location" == "https://discord.gg/CpAPpXrgUq"

GET http://localhost:3000
HOST: coreyja.tube
HTTP 303
[Asserts]
header "Location" == "https://coreyja.com/videos"
