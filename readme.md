### wot - wrapper over testops

CLI-утилита, с помощью которой можно выполнять некоторые действия в TestOps через терминал.
Цель проекта - изучение `Rust` и автоматизация рутинных задач.

### Установка

1) Необходимо [установить](https://rustup.rs/) `Rust` и `Cargo`.

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2) Перезагрузите терминал.
3) Убедитесь, что `Rust` и `Cargo` установлены.

```shell
rustc --version
cargo --version
```

4) Установите проект, выполнив команду:

```shell
cargo install --git https://github.com/VS89/plugin_testops.git
```

После установки в домашней директории `~/.config/wot/config.json` будет записан файл конфигурации.

### Использование

После установки введите в терминале:

```shell
wot
```

И следуйте инструкциям.
Для корректной работы приложения вам необходимо будет добавить API ключ от TestOps
([как его создать](https://qatools.ru/docs/overview/user-menu/)) и
endpoint развернутного Allure TestOps.

Пример загрузки локального отчета в TestOps:

```shell
wot report -d <directory_name> -p <project_id>
```

В результате потребуется подтвердить загрузку в проект:

```shell
You want to load a report into a project: '<project_name>' [y/n]? y
```

После этого будет выведена ссылка на загруженный запуск:

```shell
Link to downloaded lunch: <allure_testops_endpoint>/launch/1111
```


### ToDo

- [x] Загрузка отчета
- [ ] Конвертация из дефекта в тест-план
- [ ] Создание тест-плана по переданному списку `testcase_id`
- [ ] Запуск тест-плана по id
- [ ] Конвертация тест-кейса в файл `*.py`
