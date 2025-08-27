# Auth-Proxy-GL

**Auth-Proxy-GL** — прокси для взаимодействия между [Authlib Injector](https://github.com/yushijinhun/authlib-injector/) и [Gravit Launcher](https://gravitlauncher.com/).

---

## Установка и настройка

### 1. Скачивание
Перейдите на страницу [релизов GitHub](https://github.com/IXLShizua/auth-proxy-gl/releases/latest) и загрузите актуальную версию под вашу операционную систему.

### 2. Первый запуск
Запустите прокси. При первом старте автоматически создаётся конфигурационный файл `config.json`.

### 3. Настройка `config.json`
Пример минимальной конфигурации:

```json
{
  "binds": {
    "host": "127.0.0.1",
    "port": 10000
  },
  "servers": []
}
```

Пример рабочей конфигурации:

```json
{
  "binds": {
    "host": "127.0.0.1",
    "port": 10000
  },
  "servers": [
    {
      "name": "MyMinecraftServer",
      "api": "wss://launchserver.example.com/api",
      "token": "eyJhbGciOiJFUzI1NiJ9...",
      "meta": {
        "assets": {
          "skins": ["skins.example.com"],
          "capes": ["capes.example.com"]
        }
      }
    }
  ]
}
```

---

## Параметры конфигурации

### `binds` — параметры прокси
- **`host`** — IP-адрес, на котором работает прокси (для локального запуска `127.0.0.1`).
- **`port`** — порт, на котором работает прокси.

### `servers` — список серверов
Каждый сервер описывается объектом:
- **`name`** — имя сервера (например, `MyMinecraftServer`).
- **`api`** — WebSocket URL API лаунч-сервера (например, `ws://127.0.0.1:9274/api`).
- **`token`** — токен для аутентификации.
- **`meta.assets`** — ссылки на текстуры.

Форматы хранения текстур:
- **Объединённый формат** — все ресурсы (скины, плащи) отдаются с одного домена:
  ```json
  "assets": ["resources.example.com"]
  ```
  Используется, если сервер хранит все текстуры в одном месте.

- **Раздельный формат** — отдельные домены для разных типов ресурсов:
  ```json
  "assets": {
    "skins": ["skins.example.com"],
    "capes": ["capes.example.com"]
  }
  ```
  Используется, если текстуры разделены по типам (разные хранилища для скинов и плащей).

---

## Настройка Authlib-Injector

### Установка
1. Скачайте [последнюю версию](https://github.com/yushijinhun/authlib-injector/releases/latest).
2. Поместите `authlib-injector.jar` в папку с сервером.

### Запуск сервера
Используйте следующую команду:

```bash
java -javaagent:authlib-injector.jar=http://127.0.0.1:10000/MyMinecraftServer -jar server.jar
```

Разбор:
- `-javaagent:authlib-injector.jar` — путь к **Authlib-Injector**.
- `http://127.0.0.1:10000/MyMinecraftServer` — адрес прокси:
    - `127.0.0.1` — IP из `config.json`.
    - `10000` — порт из `config.json`.
    - `MyMinecraftServer` — имя сервера из `config.json`.
- `-jar server.jar` — запуск Minecraft-сервера.

---

## Известные проблемы

- Скины могут не обновляться из-за особенностей Mojang API, если в Gravit LaunchServer используется провайдер текстур со статическими именами (`username` или `id`).\
**Решение**: используйте провайдеры, выдающие текстуры по хешам, например [microwin7/GravitLauncher-TextureProvider](https://github.com/microwin7/GravitLauncher-TextureProvider).  
