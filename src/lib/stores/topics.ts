import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { Topic, TopicWithCountries } from '$lib/types';

// Topics state
export const topics = writable<TopicWithCountries[]>([]);
export const currentTopic = writable<TopicWithCountries | null>(null);
export const isLoadingTopics = writable<boolean>(false);

// Derived: Pinned topics
export const pinnedTopics = derived(topics, ($topics) => 
  $topics.filter((t) => t.pinned)
);

// Derived: Recent topics (last 5, sorted by updated_at)
export const recentTopics = derived(topics, ($topics) => {
  const sorted = [...$topics].sort((a, b) => 
    b.updated_at.localeCompare(a.updated_at)
  );
  return sorted.slice(0, 5);
});

/**
 * Load all topics
 */
export async function loadTopics(): Promise<void> {
  isLoadingTopics.set(true);
  try {
    const allTopics = await invoke<TopicWithCountries[]>('get_all_topics');
    topics.set(allTopics);
  } catch (error) {
    console.error('Failed to load topics:', error);
    topics.set([]);
  } finally {
    isLoadingTopics.set(false);
  }
}

/**
 * Load a specific topic
 */
export async function loadTopic(topicId: string): Promise<void> {
  isLoadingTopics.set(true);
  try {
    const topic = await invoke<TopicWithCountries>('get_topic', { topicId });
    currentTopic.set(topic);
  } catch (error) {
    console.error('Failed to load topic:', error);
    currentTopic.set(null);
  } finally {
    isLoadingTopics.set(false);
  }
}

/**
 * Create a new topic
 */
export async function createTopic(
  title: string,
  summary: string | undefined,
  color: string | undefined,
  countrySlugs: string[]
): Promise<Topic> {
  try {
    const topic = await invoke<Topic>('create_topic', {
      title,
      summary: summary || null,
      color: color || null,
      countrySlugs,
    });
    
    // Reload topics
    await loadTopics();
    
    return topic;
  } catch (error) {
    console.error('Failed to create topic:', error);
    throw error;
  }
}

/**
 * Update a topic
 */
export async function updateTopic(
  topicId: string,
  title: string,
  summary: string | undefined,
  color: string | undefined,
  pinned: boolean
): Promise<void> {
  try {
    await invoke('update_topic', {
      topicId,
      title,
      summary: summary || null,
      color: color || null,
      pinned,
    });
    
    // Reload topics
    await loadTopics();
  } catch (error) {
    console.error('Failed to update topic:', error);
    throw error;
  }
}

/**
 * Delete a topic
 */
export async function deleteTopic(topicId: string): Promise<void> {
  try {
    await invoke('delete_topic', { topicId });
    
    // Reload topics
    await loadTopics();
  } catch (error) {
    console.error('Failed to delete topic:', error);
    throw error;
  }
}

/**
 * Add a country to a topic
 */
export async function addCountryToTopic(
  topicId: string,
  countrySlug: string
): Promise<void> {
  try {
    await invoke('add_country_to_topic', { topicId, countrySlug });
    
    // Reload topics
    await loadTopics();
  } catch (error) {
    console.error('Failed to add country to topic:', error);
    throw error;
  }
}

/**
 * Remove a country from a topic
 */
export async function removeCountryFromTopic(
  topicId: string,
  countrySlug: string
): Promise<void> {
  try {
    await invoke('remove_country_from_topic', { topicId, countrySlug });
    
    // Reload topics
    await loadTopics();
  } catch (error) {
    console.error('Failed to remove country from topic:', error);
    throw error;
  }
}

/**
 * Get all topics for a country
 */
export async function getTopicsForCountry(countrySlug: string): Promise<Topic[]> {
  try {
    return await invoke<Topic[]>('get_topics_for_country', { countrySlug });
  } catch (error) {
    console.error('Failed to get topics for country:', error);
    return [];
  }
}