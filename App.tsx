import { StatusBar } from "expo-status-bar";
import { StyleSheet, Text, View, TextInput, Button } from "react-native";
import { calculateFee } from "./modules/my-rust-module";
import { useState } from "react";

export default function App() {
  const [nTxin, setNTxin] = useState("");
  const [nTxout, setNTxout] = useState("");
  const [nSpend, setNSpend] = useState("");
  const [nSout, setNSout] = useState("");
  const [value, setValue] = useState<null | number>(null);
  const [error, setError] = useState<string | null>(null);

  const handleCalculateFee = async () => {
    try {
      // Validate all inputs are non-empty and numeric
      if (!nTxin || !nTxout || !nSpend || !nSout) {
        setError("All fields are required");
        return;
      }

      const inputs = {
        nTxin: parseInt(nTxin, 10),
        nTxout: parseInt(nTxout, 10),
        nSpend: parseInt(nSpend, 10),
        nSout: parseInt(nSout, 10),
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
      setValue(fee);
      setError(null);
    } catch (err) {
      console.error("Error calculating fee:", err);
      setError(err instanceof Error ? err.message : "Unknown error occurred");
      setValue(null);
    }
  };

  return (
    <View style={styles.container}>
      <Text style={styles.title}>Transaction Fee Calculator</Text>
      {/* Input for n_txin */}
      <TextInput
        style={styles.input}
        placeholder="n_txin"
        value={nTxin}
        onChangeText={setNTxin}
        keyboardType="numeric"
      />
      {/* Input for n_txout */}
      <TextInput
        style={styles.input}
        placeholder="n_txout"
        value={nTxout}
        onChangeText={setNTxout}
        keyboardType="numeric"
      />
      {/* Input for n_spend */}
      <TextInput
        style={styles.input}
        placeholder="n_spend"
        value={nSpend}
        onChangeText={setNSpend}
        keyboardType="numeric"
      />
      {/* Input for n_sout */}
      <TextInput
        style={styles.input}
        placeholder="n_sout"
        value={nSout}
        onChangeText={setNSout}
        keyboardType="numeric"
      />
      <Button title="Calculate Fee" onPress={handleCalculateFee} />

      {error && <Text style={styles.error}>{error}</Text>}

      <Text style={styles.result}>
        {value === null ? "Result will appear here..." : `Fee: ${value}`}
      </Text>
      <StatusBar style="auto" />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: "#FFFFFF",
    alignItems: "center",
    justifyContent: "center",
    padding: 20,
  },
  title: {
    fontSize: 28,
    marginBottom: 20,
  },
  input: {
    width: "80%",
    height: 40,
    padding: 8,
    borderWidth: 1,
    borderColor: "#CCC",
    marginBottom: 10,
    borderRadius: 5,
  },
  result: {
    marginTop: 20,
    fontSize: 24,
  },
  error: {
    color: "red",
    marginTop: 10,
  },
});
