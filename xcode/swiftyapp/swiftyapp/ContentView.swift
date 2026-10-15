//
//  ContentView.swift
//  swiftyapp
//
//  Created by Jonathan McKenzie on 7/9/24.
//

import SwiftUI

struct ContentView: View {
    @Environment(\.colorScheme) private var colorScheme
    @State private var firstValue = 10
    @State private var secondValue = 32
    @State private var peerId = ""
    @State private var isStartingNode = false
    @State private var listeningAddresses = [String]()
    @State private var identityResult = ""
    @State private var connectAddress = ""
    @State private var connectResult = ""
    @State private var bootstrapResult = ""
    @State private var node: IpfsNode?
    @State private var addName = "hello.txt"
    @State private var addText = "Hello from Swift + Rust IPFS"
    @State private var addResult = ""
    @State private var isAdding = false
    @State private var catPath = ""
    @State private var catResult = ""
    @State private var isCatting = false

    private var sum: Int {
        Int(rustAdd(a: UInt32(firstValue), b: UInt32(secondValue)))
    }

    var body: some View {
        ZStack {
            LinearGradient(
                colors: [
                    colorScheme == .dark
                        ? Color(red: 0.05, green: 0.05, blue: 0.07)
                        : Color(red: 0.95, green: 0.96, blue: 0.98),
                    colorScheme == .dark
                        ? Color(red: 0.10, green: 0.08, blue: 0.06)
                        : Color(red: 0.90, green: 0.92, blue: 0.96),
                    colorScheme == .dark
                        ? Color(red: 0.18, green: 0.09, blue: 0.03)
                        : Color(red: 0.82, green: 0.86, blue: 0.93)
                ],
                startPoint: .topLeading,
                endPoint: .bottomTrailing
            )
            .ignoresSafeArea()

            if colorScheme == .dark {
                RadialGradient(
                    colors: [
                        Color(red: 1.0, green: 0.60, blue: 0.15).opacity(0.20),
                        .clear
                    ],
                    center: .topTrailing,
                    startRadius: 20,
                    endRadius: 340
                )
                .ignoresSafeArea()
            } else {
                RadialGradient(
                    colors: [
                        Color(red: 0.77, green: 0.86, blue: 1.0).opacity(0.34),
                        .clear
                    ],
                    center: .topTrailing,
                    startRadius: 24,
                    endRadius: 360
                )
                .ignoresSafeArea()
            }

            ScrollView {
                VStack(alignment: .leading, spacing: 18) {
                    header

                    glassCard {
                        HStack(alignment: .center, spacing: 16) {
                            Image(colorScheme == .dark ? "RustOrb" : "RustOrbLight")
                                .resizable()
                                .scaledToFit()
                                .frame(width: 84, height: 84)
                                .padding(8)
                                .background(
                                    colorScheme == .dark
                                        ? .white.opacity(0.04)
                                        : .white.opacity(0.70),
                                    in: RoundedRectangle(cornerRadius: 22, style: .continuous)
                                )
                                .overlay(
                                    RoundedRectangle(cornerRadius: 22, style: .continuous)
                                        .stroke(
                                            colorScheme == .dark
                                                ? Color(red: 1.0, green: 0.66, blue: 0.24).opacity(0.28)
                                                : Color(red: 0.52, green: 0.62, blue: 0.72).opacity(0.22),
                                            lineWidth: 1
                                        )
                                )

                            VStack(alignment: .leading, spacing: 10) {
                                Label("Rust bridge", systemImage: "sparkles")
                                    .font(.headline)
                                    .foregroundStyle(accentText)

                                Text(rustHello())
                                    .font(.title2.weight(.semibold))
                                    .foregroundStyle(primaryText)

                                Text("SwiftUI talking to Rust, dressed up in the same warm palette as the icon.")
                                    .font(.subheadline)
                                    .foregroundStyle(primaryText.opacity(0.74))
                            }
                        }
                    }

                    glassCard {
                        VStack(alignment: .leading, spacing: 16) {
                            HStack {
                                Label("Live calculator", systemImage: "function")
                                    .font(.headline)
                                    .foregroundStyle(accentText)
                                Spacer()
                                Text("Rust powered")
                                    .font(.caption.weight(.semibold))
                                    .padding(.horizontal, 10)
                                    .padding(.vertical, 6)
                                    .background(accentFill.opacity(colorScheme == .dark ? 0.18 : 0.12), in: Capsule())
                                    .foregroundStyle(accentText)
                            }

                            stepperRow(
                                title: "First value",
                                value: $firstValue,
                                range: 0...100
                            )

                            stepperRow(
                                title: "Second value",
                                value: $secondValue,
                                range: 0...100
                            )

                            Divider()
                                .overlay(accentFill.opacity(colorScheme == .dark ? 0.22 : 0.16))

                            HStack(alignment: .firstTextBaseline) {
                                VStack(alignment: .leading, spacing: 4) {
                                    Text("Result")
                                        .font(.caption.weight(.semibold))
                                        .foregroundStyle(primaryText.opacity(0.68))
                                    Text("\(firstValue) + \(secondValue)")
                                        .font(.title3.weight(.semibold))
                                        .foregroundStyle(primaryText)
                                }

                                Spacer()

                                Text("\(sum)")
                                    .font(.system(size: 42, weight: .bold, design: .rounded))
                                    .foregroundStyle(
                                        LinearGradient(
                                            colors: colorScheme == .dark
                                                ? [.white, Color(red: 1.0, green: 0.76, blue: 0.42)]
                                                : [Color(red: 0.10, green: 0.16, blue: 0.24), Color(red: 0.38, green: 0.45, blue: 0.58)],
                                            startPoint: .top,
                                            endPoint: .bottom
                                        )
                                    )
                            }

                            Button {
                                firstValue = Int.random(in: 0...100)
                                secondValue = Int.random(in: 0...100)
                            } label: {
                                Label("Randomize values", systemImage: "dice.fill")
                                    .frame(maxWidth: .infinity)
                            }
                            .buttonStyle(PrimaryButtonStyle())
                        }
                    }

                    glassCard {
                        VStack(alignment: .leading, spacing: 14) {
                            Label("Real IPFS node", systemImage: "network")
                                .font(.headline)
                                .foregroundStyle(accentText)

                            Button {
                                isStartingNode = true
                                Task {
                                    defer { isStartingNode = false }
                                    let created = await Task.detached { IpfsNode() }.value
                                    node = created
                                    peerId = created.peerId()
                                    listeningAddresses = created.listeningAddresses()
                                    identityResult = created.identityJson()
                                }
                            } label: {
                                Label(isStartingNode ? "Starting..." : (node == nil ? "Start IPFS node" : "Restart IPFS node"), systemImage: "play.fill")
                                    .frame(maxWidth: .infinity)
                            }
                            .buttonStyle(PrimaryButtonStyle())
                            .disabled(isStartingNode)

                            if !peerId.isEmpty {
                                Text("Peer ID: \(peerId)")
                                    .font(.footnote.monospaced())
                                    .foregroundStyle(primaryText.opacity(0.8))
                                    .textSelection(.enabled)
                            }

                            if !listeningAddresses.isEmpty {
                                VStack(alignment: .leading, spacing: 4) {
                                    Text("Listening addresses")
                                        .font(.caption.weight(.semibold))
                                        .foregroundStyle(primaryText.opacity(0.68))
                                    ForEach(listeningAddresses, id: \.self) { address in
                                        Text(address)
                                            .font(.caption.monospaced())
                                            .foregroundStyle(primaryText.opacity(0.72))
                                            .textSelection(.enabled)
                                    }
                                }
                            }

                            if !identityResult.isEmpty {
                                Text(identityResult)
                                    .font(.caption.monospaced())
                                    .foregroundStyle(primaryText.opacity(0.72))
                                    .textSelection(.enabled)
                            }

                            Button {
                                guard let node else { return }
                                Task {
                                    bootstrapResult = await Task.detached { node.defaultBootstrap() }.value
                                }
                            } label: {
                                Label("Default bootstrap", systemImage: "antenna.radiowaves.left.and.right")
                                    .frame(maxWidth: .infinity)
                            }
                            .buttonStyle(PrimaryButtonStyle())
                            .disabled(node == nil)

                            if !bootstrapResult.isEmpty {
                                Text(bootstrapResult)
                                    .font(.caption.monospaced())
                                    .foregroundStyle(primaryText.opacity(0.72))
                                    .textSelection(.enabled)
                                    .lineLimit(6)
                            }

                            HStack {
                                TextField("Multiaddr to connect", text: $connectAddress)
                                    .textFieldStyle(.roundedBorder)
                                Button("Connect") {
                                    guard let node else { return }
                                    let address = connectAddress
                                    Task {
                                        connectResult = await Task.detached { node.connect(address: address) }.value
                                    }
                                }
                                .buttonStyle(PrimaryButtonStyle())
                                .disabled(node == nil || connectAddress.isEmpty)
                            }

                            if !connectResult.isEmpty {
                                Text(connectResult)
                                    .font(.caption.monospaced())
                                    .foregroundStyle(primaryText.opacity(0.72))
                                    .textSelection(.enabled)
                            }

                            Divider()
                                .overlay(accentFill.opacity(colorScheme == .dark ? 0.22 : 0.16))

                            HStack {
                                TextField("Name", text: $addName)
                                    .textFieldStyle(.roundedBorder)
                                TextField("Text", text: $addText)
                                    .textFieldStyle(.roundedBorder)
                            }
                            Button {
                                guard let node else { return }
                                isAdding = true
                                Task {
                                    defer { isAdding = false }
                                    do {
                                        addResult = try await Task.detached { try node.addBytes(name: addName, data: Data(addText.utf8)) }.value
                                    } catch {
                                        addResult = "Add failed: \(error.localizedDescription)"
                                    }
                                }
                            } label: {
                                Label(isAdding ? "Adding..." : "Add bytes", systemImage: "plus.circle.fill")
                                    .frame(maxWidth: .infinity)
                            }
                            .buttonStyle(PrimaryButtonStyle())
                            .disabled(node == nil || isAdding || addName.isEmpty)

                            if !addResult.isEmpty {
                                Text("Added: \(addResult)")
                                    .font(.footnote.monospaced())
                                    .foregroundStyle(primaryText.opacity(0.8))
                                    .textSelection(.enabled)
                            }

                            HStack {
                                TextField("IPFS path / CID", text: $catPath)
                                    .textFieldStyle(.roundedBorder)
                                Button {
                                    guard let node else { return }
                                    isCatting = true
                                    Task {
                                        defer { isCatting = false }
                                        do {
                                            let data = try await Task.detached { try node.cat(path: catPath) }.value
                                            catResult = String(data: data, encoding: .utf8) ?? data.map { String(format: "%02x", $0) }.joined()
                                        } catch {
                                            catResult = "Cat failed: \(error.localizedDescription)"
                                        }
                                    }
                                } label: {
                                    Label(isCatting ? "Reading..." : "Cat", systemImage: "doc.text.magnifyingglass")
                                }
                                .buttonStyle(PrimaryButtonStyle())
                                .disabled(node == nil || isCatting || catPath.isEmpty)
                            }

                            if !catResult.isEmpty {
                                Text("Result: \(catResult)")
                                    .font(.footnote.monospaced())
                                    .foregroundStyle(primaryText.opacity(0.8))
                                    .textSelection(.enabled)
                                    .lineLimit(8)
                            }
                        }
                    }

                    glassCard {
                        VStack(alignment: .leading, spacing: 10) {
                            Label("What this proves", systemImage: "checkmark.seal.fill")
                                .font(.headline)
                                .foregroundStyle(accentText)

                            ForEach([
                                "SwiftUI rendering",
                                "State-driven interactions",
                                "Native Rust function calls",
                            ], id: \.self) { item in
                                HStack(spacing: 10) {
                                    Image(systemName: "checkmark.circle.fill")
                                        .foregroundStyle(accentFill)
                                    Text(item)
                                        .foregroundStyle(primaryText.opacity(colorScheme == .dark ? 0.88 : 0.84))
                                    Spacer()
                                }
                                .font(.subheadline)
                            }
                        }
                    }
                }
                .padding(20)
            }
        }
    }

    private var header: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack(alignment: .top) {
                VStack(alignment: .leading, spacing: 8) {
                    Text("Swifty Rust")
                        .font(.system(size: 34, weight: .bold, design: .rounded))
                        .foregroundStyle(primaryText)

                    Text("A polished SwiftUI shell with the same warm tone as the new icon.")
                        .font(.subheadline)
                        .foregroundStyle(primaryText.opacity(0.74))
                }

                Spacer()

                Image(colorScheme == .dark ? "RustOrb" : "RustOrbLight")
                    .resizable()
                    .scaledToFit()
                    .frame(width: 76, height: 76)
                    .padding(12)
                    .background(
                        colorScheme == .dark
                            ? .white.opacity(0.04)
                            : .white.opacity(0.78),
                        in: RoundedRectangle(cornerRadius: 22, style: .continuous)
                    )
                    .overlay(
                        RoundedRectangle(cornerRadius: 22, style: .continuous)
                            .stroke(
                                colorScheme == .dark
                                    ? Color(red: 1.0, green: 0.66, blue: 0.24).opacity(0.26)
                                    : Color(red: 0.52, green: 0.62, blue: 0.72).opacity(0.22),
                                lineWidth: 1
                            )
                    )
            }

            HStack(spacing: 8) {
                pill(text: "SwiftUI")
                pill(text: "UniFFI")
                pill(text: "Rust")
            }
        }
        .padding(.bottom, 4)
    }

    private func pill(text: String) -> some View {
        Text(text)
            .font(.caption.weight(.semibold))
            .padding(.horizontal, 12)
            .padding(.vertical, 7)
            .background(accentFill.opacity(colorScheme == .dark ? 0.14 : 0.10), in: Capsule())
            .foregroundStyle(accentText)
    }

    private func glassCard<Content: View>(@ViewBuilder content: () -> Content) -> some View {
        content()
            .padding(18)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(cardBackground, in: RoundedRectangle(cornerRadius: 24, style: .continuous))
            .overlay(
                RoundedRectangle(cornerRadius: 24, style: .continuous)
                    .stroke(accentFill.opacity(colorScheme == .dark ? 0.20 : 0.14), lineWidth: 1)
            )
            .shadow(color: .black.opacity(colorScheme == .dark ? 0.28 : 0.12), radius: 18, x: 0, y: 10)
    }

    private func stepperRow(title: String, value: Binding<Int>, range: ClosedRange<Int>) -> some View {
        HStack {
            VStack(alignment: .leading, spacing: 4) {
                Text(title)
                    .font(.subheadline.weight(.semibold))
                    .foregroundStyle(primaryText.opacity(0.92))
                Text("Tap +/- or use the randomizer")
                    .font(.caption)
                    .foregroundStyle(primaryText.opacity(0.62))
            }

            Spacer()

            Stepper(value: value, in: range) {
                Text("\(value.wrappedValue)")
                    .font(.title3.weight(.semibold))
                    .monospacedDigit()
                    .foregroundStyle(primaryText)
                    .frame(minWidth: 44, alignment: .trailing)
            }
            .labelsHidden()
            .tint(accentFill)
        }
    }

    private var primaryText: Color {
        colorScheme == .dark ? .white : Color(red: 0.10, green: 0.14, blue: 0.20)
    }

    private var accentFill: Color {
        colorScheme == .dark ? Color(red: 1.0, green: 0.66, blue: 0.24) : Color(red: 0.82, green: 0.38, blue: 0.10)
    }

    private var accentText: Color {
        colorScheme == .dark ? Color(red: 1.0, green: 0.86, blue: 0.62) : Color(red: 0.42, green: 0.24, blue: 0.12)
    }

    private var cardBackground: Color {
        colorScheme == .dark ? .white.opacity(0.05) : .white.opacity(0.76)
    }
}

private struct PrimaryButtonStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(.headline.weight(.semibold))
            .padding(.vertical, 14)
            .foregroundStyle(.white)
            .background(
                LinearGradient(
                    colors: [
                        Color(red: 1.0, green: 0.56, blue: 0.12),
                        Color(red: 0.93, green: 0.30, blue: 0.08)
                    ],
                    startPoint: .leading,
                    endPoint: .trailing
                ),
                in: RoundedRectangle(cornerRadius: 16, style: .continuous)
            )
            .scaleEffect(configuration.isPressed ? 0.98 : 1.0)
            .opacity(configuration.isPressed ? 0.92 : 1.0)
    }
}

#Preview {
    ContentView()
}
