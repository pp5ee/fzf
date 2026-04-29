#!/usr/bin/env ruby
# frozen_string_literal: true

# Simplified verification script for Rust fzf implementation
# Does not require bundler

require 'minitest/autorun'
require 'fileutils'
require 'tempfile'

BASE = File.expand_path('..', __dir__)
FZF = "#{BASE}/bin/fzf"

class TestRustFzf < Minitest::Test
  def setup
    skip "fzf binary not found" unless File.exist?(FZF)
  end

  def test_version
    result = `#{FZF} --version`
    assert_equal 0, $?.exitstatus
    assert_match(/^\d+\.\d+\.\d+/, result)
  end

  def test_help
    result = `#{FZF} --help`
    assert_equal 0, $?.exitstatus
    assert_includes result, "fzf"
  end

  def test_filter_basic
    result = `echo "test" | #{FZF} -f "test"`
    assert_equal 0, $?.exitstatus
    assert_equal "test", result.chomp
  end

  def test_filter_no_match
    result = `echo "foo" | #{FZF} -f "bar" 2>&1`
    assert_equal 1, $?.exitstatus
  end

  def test_filter_exact
    # Test exact matching - use printf for portability
    result = `printf "%s\n" "test" "Test" | #{FZF} -f "test" -e`
    lines = result.lines(chomp: true)
    # Both should match (case insensitive by default)
    assert_includes lines, "test"
    assert_includes lines, "Test"
  end

  def test_shell_integration_bash
    result = `#{FZF} --bash`
    assert_equal 0, $?.exitstatus
    assert_includes result, "key-bindings.bash"
  end

  def test_shell_integration_zsh
    result = `#{FZF} --zsh`
    assert_equal 0, $?.exitstatus
    assert_includes result, "key-bindings.zsh"
  end

  def test_shell_integration_fish
    result = `#{FZF} --fish`
    assert_equal 0, $?.exitstatus
    assert_includes result, "key-bindings.fish"
  end

  def test_filter_fuzzy
    # Fuzzy matching should match partial patterns
    result = `printf "%s\n" "apple" "banana" "cherry" | #{FZF} -f "an"`
    lines = result.lines(chomp: true)
    assert_includes lines, "banana"
    refute_includes lines, "apple"
  end

  def test_filter_exit_codes
    # Match should exit 0
    `echo "test" | #{FZF} -f "test"`
    assert_equal 0, $?.exitstatus

    # No match should exit 1
    `echo "foo" | #{FZF} -f "bar" 2>&1`
    assert_equal 1, $?.exitstatus
  end

  def test_extended_mode
    # Extended mode with prefix
    result = `printf "%s\n" "test123" "123test" | #{FZF} -x -f "^test"`
    lines = result.lines(chomp: true)
    assert_includes lines, "test123"
    refute_includes lines, "123test"
  end
end
