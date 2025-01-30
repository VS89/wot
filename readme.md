### wot - wrapper over testops

CLI утилита, с помощью которой можно выполнять некоторые действия в тестопсе через терминал.
Цель проекта - изучение языка программирования `rust` и автоматизация рутинных задач

### Установка

1) Неободимо [установить](https://rustup.rs/) `Rust` и `Cargo`

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2) Перезагрузить терминал
3) Убедиться, что `Rust` и `Cargo` установились

```shell
rustc --version
cargo --version
```

4) Установить проект выполнив команду

```shell
cargo install --git https://github.com/VS89/plugin_testops.git
```

### Использование

После установки введите в терминале:

```shell
wot
```

и следуйте инструкциям.
Для корректной работы приложения вам нужно будет добавить API key от TestOps
([как его создать](https://qatools.ru/docs/overview/user-menu/)) и
endpoint развернутного Allure TestOps.

Пример загрузки локального отчета в тестопс:

```shell
wot report -d <directory_name> -p <project_id>
```

В результате нужно будет подтвердить загрузку в проект

```shell
You want to load a report into a project: '<project_name>' [y/n]? y
```

После чего будет вывелена ссылка на зугруженный лаунч

```shell
Link to downloaded lunch: <allure_testops_endpoint>/launch/1111
```


### ToDo

- [x] Загрузка отчета
- [ ] Конвертация из дефекта в тестплан
- [ ] Создание тестплана по переданному списку `testcase_id`
- [ ] Запуск тестплана по id
- [ ] Конвертация тесткейса в файл `*.py`
