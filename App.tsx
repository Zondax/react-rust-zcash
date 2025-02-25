import { StatusBar } from "expo-status-bar";
import {
  StyleSheet,
  Text,
  View,
  TextInput,
  Button,
  ScrollView,
  TouchableOpacity,
  Alert,
} from "react-native";
import {
  calculateFee,
  createBuilder,
  destroyBuilder,
  addTransparentInput,
  addTransparentOutput,
  buildTransaction,
  getErrorDescription,
} from "./modules/my-rust-module";
import { useState } from "react";

export default function App() {
  // Transaction fee calculator states
  const [nTxin, setNTxin] = useState("");
  const [nTxout, setNTxout] = useState("");
  const [nSpend, setNSpend] = useState("");
  const [nSout, setNSout] = useState("");
  const [calculatedFee, setCalculatedFee] = useState(null);
  const [error, setError] = useState(null);

  // Transaction builder states
  const [builderId, setBuilderId] = useState(null);
  const [networkType, setNetworkType] = useState("0"); // 0 for mainnet
  const [currentScreen, setCurrentScreen] = useState("feeCalculator");

  // Transaction inputs states
  const [inputs, setInputs] = useState([
    {
      outp: "",
      pk: "",
      address: "",
      value: "",
    },
  ]);

  // Transaction outputs states
  const [outputs, setOutputs] = useState([
    {
      address: "",
      value: "",
    },
  ]);

  // Transaction build result
  const [txResult, setTxResult] = useState(null);

  // Constants (these would typically come from configuration)
  const SPEND_PATH = "/path/to/spend/params"; // Hardcoded path
  const OUTPUT_PATH = "/path/to/output/params"; // Hardcoded path
  const TX_VERSION = 5; // Default transaction version for Zcash

  // Calculate fee handler
  const handleCalculateFee = async () => {
    try {
      // Validate all inputs for empty ones we default to 0
      const inputs = {
        nTxin: nTxin ? parseInt(nTxin, 10) : 0,
        nTxout: nTxout ? parseInt(nTxout, 10) : 0,
        nSpend: nSpend ? parseInt(nSpend, 10) : 0,
        nSout: nSout ? parseInt(nSout, 10) : 0,
      };

      // Validate all parsed values are valid numbers
      if (Object.values(inputs).some(isNaN)) {
        setError("All fields must be valid numbers");
        return;
      }

      // Validate all values are non-negative
      if (Object.values(inputs).some((val) => val < 0)) {
        setError("All values must be non-negative");
        return;
      }

      console.log("Calculating fee with inputs:", inputs);
      const fee = await calculateFee(
        inputs.nTxin,
        inputs.nTxout,
        inputs.nSpend,
        inputs.nSout,
      );

      console.log("Fee calculation result:", fee);
      setCalculatedFee(fee);
      setError(null);
    } catch (err) {
      console.error("Error calculating fee:", err);
      setError(err instanceof Error ? err.message : "Unknown error occurred");
      setCalculatedFee(null);
    }
  };

  // Create builder handler
  const handleCreateBuilder = async () => {
    if (calculatedFee === null) {
      setError("Please calculate a fee first");
      return;
    }

    try {
      const networkTypeInt = parseInt(networkType, 10);
      if (isNaN(networkTypeInt)) {
        setError("Network type must be a valid number");
        return;
      }

      const id = await createBuilder(calculatedFee, networkTypeInt);
      if (id === -1) {
        setError("Failed to create builder");
        return;
      }

      setBuilderId(id);
      setError(null);
      setCurrentScreen("inputsOutputs");
    } catch (err) {
      console.error("Error creating builder:", err);
      setError(err instanceof Error ? err.message : "Unknown error occurred");
    }
  };

  // Add input field handler
  const addInputField = () => {
    setInputs([...inputs, { outp: "", pk: "", address: "", value: "" }]);
  };

  // Add output field handler
  const addOutputField = () => {
    setOutputs([...outputs, { address: "", value: "" }]);
  };

  // Update input field handler
  const updateInputField = (index, field, value) => {
    const newInputs = [...inputs];
    newInputs[index] = { ...newInputs[index], [field]: value };
    setInputs(newInputs);
  };

  // Update output field handler
  const updateOutputField = (index, field, value) => {
    const newOutputs = [...outputs];
    newOutputs[index] = { ...newOutputs[index], [field]: value };
    setOutputs(newOutputs);
  };

  // Remove input field handler
  const removeInputField = (index) => {
    if (inputs.length > 1) {
      const newInputs = [...inputs];
      newInputs.splice(index, 1);
      setInputs(newInputs);
    }
  };

  // Remove output field handler
  const removeOutputField = (index) => {
    if (outputs.length > 1) {
      const newOutputs = [...outputs];
      newOutputs.splice(index, 1);
      setOutputs(newOutputs);
    }
  };

  // Add transparent inputs and outputs to builder
  const addTransparentData = async () => {
    if (builderId === null) {
      setError("No active builder found");
      return;
    }

    try {
      // Add each input
      for (let i = 0; i < inputs.length; i++) {
        const input = inputs[i];

        // Validate input
        if (!input.outp || !input.pk || !input.address || !input.value) {
          setError(`Input #${i + 1} has missing fields`);
          return;
        }

        const value = parseInt(input.value, 10);
        if (isNaN(value) || value <= 0) {
          setError(`Input #${i + 1} has invalid value`);
          return;
        }

        const result = await addTransparentInput(builderId, {
          outp: input.outp,
          pk: input.pk,
          address: input.address,
          value: value,
        });

        if (result !== 0) {
          // 0 is ZcashError::Success
          const errorMsg = await getErrorDescription(result);
          setError(`Failed to add input #${i + 1}: ${errorMsg}`);
          return;
        }
      }

      // Add each output
      for (let i = 0; i < outputs.length; i++) {
        const output = outputs[i];

        // Validate output
        if (!output.address || !output.value) {
          setError(`Output #${i + 1} has missing fields`);
          return;
        }

        const value = parseInt(output.value, 10);
        if (isNaN(value) || value <= 0) {
          setError(`Output #${i + 1} has invalid value`);
          return;
        }

        const result = await addTransparentOutput(builderId, {
          address: output.address,
          value: value,
        });

        if (result !== 0) {
          // 0 is ZcashError::Success
          const errorMsg = await getErrorDescription(result);
          setError(`Failed to add output #${i + 1}: ${errorMsg}`);
          return;
        }
      }

      setError(null);
      setCurrentScreen("buildTx");
    } catch (err) {
      console.error("Error adding transparent data:", err);
      setError(err instanceof Error ? err.message : "Unknown error occurred");
    }
  };

  // Build transaction handler
  const handleBuildTransaction = async () => {
    if (builderId === null) {
      setError("No active builder found");
      return;
    }

    try {
      const result = await buildTransaction(
        builderId,
        SPEND_PATH,
        OUTPUT_PATH,
        TX_VERSION,
      );

      setTxResult(result);
      setError(null);

      // Clean up the builder
      await destroyBuilder(builderId);
      setBuilderId(null);

      // Success message
      Alert.alert(
        "Transaction Built Successfully",
        `Transaction hash: ${result.substring(0, 32)}...`,
        [{ text: "OK", onPress: () => resetForm() }],
      );
    } catch (err) {
      console.error("Error building transaction:", err);
      setError(err instanceof Error ? err.message : "Unknown error occurred");
    }
  };

  // Reset the form and go back to fee calculator
  const resetForm = () => {
    setInputs([{ outp: "", pk: "", address: "", value: "" }]);
    setOutputs([{ address: "", value: "" }]);
    setTxResult(null);
    setCurrentScreen("feeCalculator");
  };

  // Cancel builder and destroy if exists
  const cancelBuilder = async () => {
    if (builderId !== null) {
      try {
        await destroyBuilder(builderId);
      } catch (err) {
        console.error("Error destroying builder:", err);
      }
    }

    setBuilderId(null);
    resetForm();
  };

  // Render fee calculator screen
  const renderFeeCalculator = () => (
    <View style={styles.section}>
      <Text style={styles.title}>Transaction Fee Calculator</Text>

      <TextInput
        style={styles.input}
        placeholder="Number of transparent inputs"
        value={nTxin}
        onChangeText={setNTxin}
        keyboardType="numeric"
      />

      <TextInput
        style={styles.input}
        placeholder="Number of transparent outputs"
        value={nTxout}
        onChangeText={setNTxout}
        keyboardType="numeric"
      />

      <TextInput
        style={styles.input}
        placeholder="Number of shielded spends"
        value={nSpend}
        onChangeText={setNSpend}
        keyboardType="numeric"
      />

      <TextInput
        style={styles.input}
        placeholder="Number of shielded outputs"
        value={nSout}
        onChangeText={setNSout}
        keyboardType="numeric"
      />

      <TextInput
        style={styles.input}
        placeholder="Network Type (0=Mainnet, 1=Testnet)"
        value={networkType}
        onChangeText={setNetworkType}
        keyboardType="numeric"
      />

      <View style={styles.buttonContainer}>
        <Button title="Calculate Fee" onPress={handleCalculateFee} />
      </View>

      <Text style={styles.result}>
        {calculatedFee === null
          ? "Fee will appear here..."
          : `Fee: ${calculatedFee}`}
      </Text>

      {calculatedFee !== null && (
        <View style={styles.buttonContainer}>
          <Button
            title="Create Transaction Builder"
            onPress={handleCreateBuilder}
          />
        </View>
      )}
    </View>
  );

  // Render inputs and outputs screen
  const renderInputsOutputs = () => (
    <ScrollView style={styles.scrollContainer}>
      <View style={styles.section}>
        <Text style={styles.title}>Transaction Builder</Text>
        <Text style={styles.subtitle}>Builder ID: {builderId}</Text>
        <Text style={styles.subtitle}>Fee: {calculatedFee}</Text>

        <Text style={styles.sectionHeader}>Transparent Inputs</Text>
        {inputs.map((input, index) => (
          <View key={`input-${index}`} style={styles.fieldContainer}>
            <Text style={styles.fieldLabel}>Input #{index + 1}</Text>

            <TextInput
              style={styles.input}
              placeholder="Outpoint (txid+vout)"
              value={input.outp}
              onChangeText={(value) => updateInputField(index, "outp", value)}
            />

            <TextInput
              style={styles.input}
              placeholder="Public Key"
              value={input.pk}
              onChangeText={(value) => updateInputField(index, "pk", value)}
            />

            <TextInput
              style={styles.input}
              placeholder="Address"
              value={input.address}
              onChangeText={(value) =>
                updateInputField(index, "address", value)
              }
            />

            <TextInput
              style={styles.input}
              placeholder="Value (in zatoshis)"
              value={input.value}
              onChangeText={(value) => updateInputField(index, "value", value)}
              keyboardType="numeric"
            />

            <TouchableOpacity
              style={styles.removeButton}
              onPress={() => removeInputField(index)}
            >
              <Text style={styles.removeButtonText}>Remove</Text>
            </TouchableOpacity>
          </View>
        ))}

        <TouchableOpacity style={styles.addButton} onPress={addInputField}>
          <Text style={styles.addButtonText}>+ Add Another Input</Text>
        </TouchableOpacity>

        <Text style={styles.sectionHeader}>Transparent Outputs</Text>
        {outputs.map((output, index) => (
          <View key={`output-${index}`} style={styles.fieldContainer}>
            <Text style={styles.fieldLabel}>Output #{index + 1}</Text>

            <TextInput
              style={styles.input}
              placeholder="Address"
              value={output.address}
              onChangeText={(value) =>
                updateOutputField(index, "address", value)
              }
            />

            <TextInput
              style={styles.input}
              placeholder="Value (in zatoshis)"
              value={output.value}
              onChangeText={(value) => updateOutputField(index, "value", value)}
              keyboardType="numeric"
            />

            <TouchableOpacity
              style={styles.removeButton}
              onPress={() => removeOutputField(index)}
            >
              <Text style={styles.removeButtonText}>Remove</Text>
            </TouchableOpacity>
          </View>
        ))}

        <TouchableOpacity style={styles.addButton} onPress={addOutputField}>
          <Text style={styles.addButtonText}>+ Add Another Output</Text>
        </TouchableOpacity>

        <View style={styles.buttonRow}>
          <TouchableOpacity
            style={[styles.button, styles.cancelButton]}
            onPress={cancelBuilder}
          >
            <Text style={styles.buttonText}>Cancel</Text>
          </TouchableOpacity>

          <TouchableOpacity
            style={[styles.button, styles.nextButton]}
            onPress={addTransparentData}
          >
            <Text style={styles.buttonText}>Next</Text>
          </TouchableOpacity>
        </View>
      </View>
    </ScrollView>
  );

  // Render build transaction screen
  const renderBuildTx = () => (
    <View style={styles.section}>
      <Text style={styles.title}>Build Transaction</Text>
      <Text style={styles.infoText}>
        All your inputs and outputs have been added to the transaction builder.
        You can now build the final transaction.
      </Text>

      <View style={styles.summaryContainer}>
        <Text style={styles.summaryText}>Inputs: {inputs.length}</Text>
        <Text style={styles.summaryText}>Outputs: {outputs.length}</Text>
        <Text style={styles.summaryText}>
          Network: {networkType === "0" ? "Mainnet" : "Testnet"}
        </Text>
        <Text style={styles.summaryText}>Fee: {calculatedFee}</Text>
      </View>

      <View style={styles.buttonRow}>
        <TouchableOpacity
          style={[styles.button, styles.cancelButton]}
          onPress={cancelBuilder}
        >
          <Text style={styles.buttonText}>Cancel</Text>
        </TouchableOpacity>

        <TouchableOpacity
          style={[styles.button, styles.buildButton]}
          onPress={handleBuildTransaction}
        >
          <Text style={styles.buttonText}>Build Transaction</Text>
        </TouchableOpacity>
      </View>

      {txResult && (
        <View style={styles.resultContainer}>
          <Text style={styles.resultTitle}>Transaction Built!</Text>
          <Text style={styles.txData}>{txResult}</Text>
        </View>
      )}
    </View>
  );

  return (
    <View style={styles.container}>
      {error && <Text style={styles.error}>{error}</Text>}

      {currentScreen === "feeCalculator" && renderFeeCalculator()}
      {currentScreen === "inputsOutputs" && renderInputsOutputs()}
      {currentScreen === "buildTx" && renderBuildTx()}

      <StatusBar style="auto" />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: "#FFFFFF",
    padding: 20,
  },
  scrollContainer: {
    flex: 1,
  },
  section: {
    width: "100%",
    alignItems: "center",
    justifyContent: "center",
  },
  title: {
    fontSize: 24,
    fontWeight: "bold",
    marginBottom: 20,
    marginTop: 30,
  },
  subtitle: {
    fontSize: 16,
    marginBottom: 5,
  },
  sectionHeader: {
    fontSize: 18,
    fontWeight: "bold",
    marginTop: 20,
    marginBottom: 10,
    alignSelf: "flex-start",
  },
  input: {
    width: "100%",
    height: 40,
    padding: 8,
    borderWidth: 1,
    borderColor: "#CCC",
    marginBottom: 10,
    borderRadius: 5,
  },
  buttonContainer: {
    marginTop: 10,
    marginBottom: 10,
    width: "80%",
  },
  result: {
    marginTop: 20,
    fontSize: 18,
    fontWeight: "bold",
  },
  error: {
    color: "red",
    marginTop: 10,
    padding: 10,
    backgroundColor: "#FFE5E5",
    borderRadius: 5,
    width: "100%",
    textAlign: "center",
  },
  fieldContainer: {
    width: "100%",
    marginBottom: 20,
    padding: 10,
    backgroundColor: "#F5F5F5",
    borderRadius: 5,
  },
  fieldLabel: {
    fontSize: 16,
    fontWeight: "bold",
    marginBottom: 5,
  },
  addButton: {
    alignSelf: "flex-start",
    marginTop: 10,
    marginBottom: 20,
    padding: 10,
    backgroundColor: "#E0F7FA",
    borderRadius: 5,
  },
  addButtonText: {
    color: "#00838F",
    fontWeight: "bold",
  },
  removeButton: {
    alignSelf: "flex-end",
    marginTop: 5,
    padding: 8,
    backgroundColor: "#FFEBEE",
    borderRadius: 5,
  },
  removeButtonText: {
    color: "#C62828",
  },
  buttonRow: {
    flexDirection: "row",
    justifyContent: "space-between",
    width: "100%",
    marginTop: 20,
    marginBottom: 20,
  },
  button: {
    padding: 12,
    borderRadius: 5,
    width: "48%",
    alignItems: "center",
  },
  cancelButton: {
    backgroundColor: "#EEEEEE",
  },
  nextButton: {
    backgroundColor: "#2196F3",
  },
  buildButton: {
    backgroundColor: "#4CAF50",
  },
  buttonText: {
    color: "#FFFFFF",
    fontWeight: "bold",
  },
  summaryContainer: {
    width: "100%",
    marginTop: 20,
    padding: 15,
    backgroundColor: "#F5F5F5",
    borderRadius: 5,
  },
  summaryText: {
    fontSize: 16,
    marginBottom: 5,
  },
  infoText: {
    textAlign: "center",
    marginBottom: 20,
  },
  resultContainer: {
    width: "100%",
    marginTop: 20,
    padding: 15,
    backgroundColor: "#E8F5E9",
    borderRadius: 5,
  },
  resultTitle: {
    fontSize: 18,
    fontWeight: "bold",
    marginBottom: 10,
  },
  txData: {
    fontFamily: "monospace",
    fontSize: 12,
  },
});
