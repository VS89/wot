use std::collections::HashMap;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Scenario {
    root: Root,
    pub scenario_steps: HashMap<String, ScenarioStep>,
}

impl Scenario {
    #[cfg(test)]
    pub fn default() -> Self {
        let mut steps = HashMap::new();
        steps.insert("123".to_string(), ScenarioStep::default());
        Self {
            root: Root::default(),
            scenario_steps: steps,
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    children: Vec<u64>,
}

impl Root {
    #[cfg(test)]
    pub fn default() -> Self {
        Self {
            children: vec![123],
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScenarioStep {
    id: u64,
    body: String,
    #[serde(default)]
    expected_result_id: Option<u64>,
    #[serde(default)]
    children: Option<Vec<u64>>,
}

impl ScenarioStep {
    #[cfg(test)]
    pub fn default() -> Self {
        Self {
            id: 111,
            body: "111_body".to_string(),
            expected_result_id: None,
            children: None,
        }
    }
}

impl Scenario {
    pub fn get_scenario(&self) -> String {
        let mut step_strings: Vec<String> = vec![];
        for &id in &self.root.children {
            let step_key = id.to_string();
            if let Some(step) = self.scenario_steps.get(&step_key) {
                step_strings.push(step.body.clone());
                step.expected_result_id
                    .and_then(|eid| self.scenario_steps.get(&eid.to_string()))
                    .filter(|estep| estep.body == "Expected Result")
                    .and_then(|estep| estep.children.as_ref())
                    .into_iter()
                    .flatten()
                    .filter_map(|&cid| self.scenario_steps.get(&cid.to_string()))
                    .map(|child_step| format!("\t{}", child_step.body))
                    .for_each(|step| step_strings.push(step));
            }
        }
        step_strings.join("\n\t\t\t")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_scenario() -> Scenario {
        let mut steps = HashMap::new();

        steps.insert(
            "1".to_string(),
            ScenarioStep {
                id: 1,
                body: "Step 1".to_string(),
                expected_result_id: Some(2),
                children: None,
            },
        );

        steps.insert(
            "2".to_string(),
            ScenarioStep {
                id: 2,
                body: "Expected Result".to_string(),
                expected_result_id: None,
                children: Some(vec![3]),
            },
        );

        steps.insert(
            "3".to_string(),
            ScenarioStep {
                id: 3,
                body: "Substep 1".to_string(),
                expected_result_id: None,
                children: None,
            },
        );

        Scenario {
            root: Root { children: vec![1] },
            scenario_steps: steps,
        }
    }

    // Проверяем базовый сценарий с одним шагом и вложенным элементом
    #[test]
    fn test_scenario_generation_basic() {
        let scenario = create_test_scenario();
        let result = scenario.get_scenario();
        let expected = "Step 1\n\t\t\t\tSubstep 1";
        assert_eq!(result, expected);
    }

    //  Обработка нескольких шагов со вложенными элементами
    #[test]
    fn test_multiple_root_children() {
        let mut scenario = create_test_scenario();
        scenario.root.children.push(4);

        scenario.scenario_steps.insert(
            "4".to_string(),
            ScenarioStep {
                id: 4,
                body: "Step 2".to_string(),
                expected_result_id: Some(5),
                children: None,
            },
        );

        scenario.scenario_steps.insert(
            "5".to_string(),
            ScenarioStep {
                id: 5,
                body: "Expected Result".to_string(),
                expected_result_id: None,
                children: Some(vec![6]),
            },
        );

        scenario.scenario_steps.insert(
            "6".to_string(),
            ScenarioStep {
                id: 6,
                body: "Substep 2".to_string(),
                expected_result_id: None,
                children: None,
            },
        );

        let result = scenario.get_scenario();
        let expected = "Step 1\n\t\t\t\tSubstep 1\n\t\t\tStep 2\n\t\t\t\tSubstep 2";
        assert_eq!(result, expected);
    }

    // Игнорируем некорректный expected_result
    #[test]
    fn test_invalid_expected_result_ignored() {
        let mut scenario = create_test_scenario();
        scenario.scenario_steps.get_mut("2").unwrap().body = "Invalid".to_string();

        let result = scenario.get_scenario();
        assert_eq!(result, "Step 1");
    }

    // Обработка шагов без expected_result
    #[test]
    fn test_missing_expected_result() {
        let mut scenario = create_test_scenario();
        scenario
            .scenario_steps
            .get_mut("1")
            .unwrap()
            .expected_result_id = None;

        let result = scenario.get_scenario();
        assert_eq!(result, "Step 1");
    }
}
